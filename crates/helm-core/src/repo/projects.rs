//! 项目仓库：CRUD + OpLog 记录。

use rusqlite::{Connection, Row, params};

use crate::error::{HelmError, Result};
use crate::models::{
    CreateProjectInput, Project, ProjectMethod, ProjectStatus, UpdateProjectInput,
};
use crate::repo::oplog;
use crate::util;

const COLS: &str = "id, name, code, method, status, description, start_date, target_end_date, bac, created_at, updated_at";

fn map_row(row: &Row) -> rusqlite::Result<Project> {
    let method: String = row.get("method")?;
    let status: String = row.get("status")?;
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
    conn.execute(
        "INSERT INTO projects (id, name, code, method, status, description, start_date, target_end_date, bac, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, 'planning', ?5, ?6, ?7, ?8, ?9, ?9)",
        params![
            id,
            input.name.trim(),
            input.code,
            input.method,
            input.description,
            input.start_date,
            input.target_end_date,
            input.bac,
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
