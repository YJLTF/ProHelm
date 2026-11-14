//! 项目仓库：CRUD + OpLog 记录。

use rusqlite::{Connection, Row, params};

use crate::error::{HelmError, Result};
use crate::models::{
    CreateProjectInput, Project, ProjectMethod, ProjectStatus, UpdateProjectInput,
};
use crate::repo::oplog;
use crate::util;

const COLS: &str = "id, name, code, method, status, description, start_date, target_end_date, bac, tailored, created_at, updated_at";

fn map_row(row: &Row) -> rusqlite::Result<Project> {
    let method: String = row.get("method")?;
    let status: String = row.get("status")?;
    let tailored_raw: Option<String> = row.get("tailored")?;
    let tailored = match tailored_raw {
        Some(s) => {
            let parsed = serde_json::from_str::<Vec<String>>(&s).map_err(|e| {
                crate::repo::enum_conversion_error(HelmError::Validation(format!(
                    "裁剪配置损坏: {e}"
                )))
            })?;
            Some(parsed)
        }
        None => None,
    };
    Ok(Project {
        id: row.get("id")?,
        name: row.get("name")?,
        code: row.get("code")?,
        method: ProjectMethod::parse(&method).map_err(crate::repo::enum_conversion_error)?,
        status: ProjectStatus::parse(&status).map_err(crate::repo::enum_conversion_error)?,
        description: row.get("description")?,
        start_date: row.get("start_date")?,
        target_end_date: row.get("target_end_date")?,
        bac: row.get("bac")?,
        tailored,
        created_at: row.get("created_at")?,
        updated_at: row.get("updated_at")?,
    })
}

fn not_found(id: &str) -> HelmError {
    HelmError::NotFound(format!("项目 {id} 不存在"))
}

pub fn list(conn: &Connection) -> Result<Vec<Project>> {
    let sql = format!("SELECT {COLS} FROM projects ORDER BY updated_at DESC");
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map([], map_row)?;
    Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
}

pub fn get(conn: &Connection, id: &str) -> Result<Project> {
    let sql = format!("SELECT {COLS} FROM projects WHERE id = ?1");
    conn.query_row(&sql, params![id], map_row)
        .map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => not_found(id),
            other => other.into(),
        })
}

pub fn create(conn: &Connection, input: &CreateProjectInput) -> Result<Project> {
    input.validate()?;
    let id = util::new_id();
    let now = util::now_iso();
    let tailored_json = serde_json::to_string(&crate::tailoring::preset_for_method(input.method))?;
    conn.execute(
        "INSERT INTO projects (id, name, code, method, status, description, start_date, target_end_date, bac, tailored, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, 'planning', ?5, ?6, ?7, ?8, ?9, ?10, ?10)",
        params![
            id,
            input.name.trim(),
            input.code,
            input.method,
            input.description,
            input.start_date,
            input.target_end_date,
            input.bac,
            tailored_json,
            now,
        ],
    )?;
    oplog::record(
        conn,
        "project",
        &id,
        "create",
        &serde_json::to_string(input)?,
    )?;
    get(conn, &id)
}

pub fn update(conn: &Connection, input: &UpdateProjectInput) -> Result<Project> {
    input.validate()?;
    let now = util::now_iso();
    let affected = conn.execute(
        "UPDATE projects SET
            name = COALESCE(?1, name),
            code = COALESCE(?2, code),
            method = COALESCE(?3, method),
            status = COALESCE(?4, status),
            description = COALESCE(?5, description),
            start_date = COALESCE(?6, start_date),
            target_end_date = COALESCE(?7, target_end_date),
            bac = COALESCE(?8, bac),
            updated_at = ?9
         WHERE id = ?10",
        params![
            input.name,
            input.code,
            input.method,
            input.status,
            input.description,
            input.start_date,
            input.target_end_date,
            input.bac,
            now,
            input.id,
        ],
    )?;
    if affected == 0 {
        return Err(not_found(&input.id));
    }
    oplog::record(
        conn,
        "project",
        &input.id,
        "update",
        &serde_json::to_string(input)?,
    )?;
    get(conn, &input.id)
}

pub fn delete(conn: &Connection, id: &str) -> Result<()> {
    let affected = conn.execute("DELETE FROM projects WHERE id = ?1", params![id])?;
    if affected == 0 {
        return Err(not_found(id));
    }
    oplog::record(conn, "project", id, "delete", "{}")
}

/// 覆盖保存方法论裁剪配置（key 必须在 tailoring::ALL_ITEMS 中）。
pub fn set_tailored(conn: &Connection, project_id: &str, keys: &[String]) -> Result<Project> {
    for k in keys {
        if !crate::tailoring::is_valid_key(k) {
            return Err(HelmError::Validation(format!("未知的流程项: {k}")));
        }
    }
    let json = serde_json::to_string(keys)?;
    let affected = conn.execute(
        "UPDATE projects SET tailored = ?1, updated_at = ?2 WHERE id = ?3",
        params![json, util::now_iso(), project_id],
    )?;
    if affected == 0 {
        return Err(not_found(project_id));
    }
    oplog::record(conn, "project", project_id, "tailor", &json)?;
    get(conn, project_id)
}

/// 重置为当前开发方法的推荐裁剪预设。
pub fn apply_recommended_tailoring(conn: &Connection, project_id: &str) -> Result<Project> {
    let project = get(conn, project_id)?;
    let preset: Vec<String> = crate::tailoring::preset_for_method(project.method)
        .into_iter()
        .map(String::from)
        .collect();
    set_tailored(conn, project_id, &preset)
}
