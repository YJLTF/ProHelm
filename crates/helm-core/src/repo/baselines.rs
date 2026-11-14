//! 计划基线仓库：任务/依赖结构快照（M2 挣值管理的对比基准）。

use rusqlite::{Connection, Row, params};

use crate::error::{HelmError, Result};
use crate::models::Baseline;
use crate::repo::{dependencies, oplog, projects, tasks};
use crate::util;

const COLS: &str = "id, project_id, name, snapshot, created_at";

fn map_row(row: &Row) -> rusqlite::Result<Baseline> {
    Ok(Baseline {
        id: row.get("id")?,
        project_id: row.get("project_id")?,
        name: row.get("name")?,
        snapshot: row.get("snapshot")?,
        created_at: row.get("created_at")?,
    })
}

fn not_found(id: &str) -> HelmError {
    HelmError::NotFound(format!("基线 {id} 不存在"))
}

pub fn list_by_project(conn: &Connection, project_id: &str) -> Result<Vec<Baseline>> {
    let sql =
        format!("SELECT {COLS} FROM baselines WHERE project_id = ?1 ORDER BY created_at DESC");
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(params![project_id], map_row)?;
    Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
}

pub fn get(conn: &Connection, id: &str) -> Result<Baseline> {
    let sql = format!("SELECT {COLS} FROM baselines WHERE id = ?1");
    conn.query_row(&sql, params![id], map_row)
        .map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => not_found(id),
            other => other.into(),
        })
}

/// 抓取当前计划结构并保存为基线。
pub fn create(conn: &Connection, project_id: &str, name: &str) -> Result<Baseline> {
    let project = projects::get(conn, project_id)?;
    if name.trim().is_empty() {
        return Err(HelmError::Validation("基线名称不能为空".into()));
    }
    let task_list = tasks::list_by_project(conn, project_id)?;
    let dep_list = dependencies::list_by_project(conn, project_id)?;
    let snapshot = serde_json::json!({
        "capturedAt": util::now_iso(),
        "projectStatus": project.status.as_str(),
        "tasks": task_list,
        "dependencies": dep_list,
    });

    let id = util::new_id();
    let now = util::now_iso();
    conn.execute(
        "INSERT INTO baselines (id, project_id, name, snapshot, created_at) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![id, project_id, name.trim(), snapshot.to_string(), now],
    )?;
    oplog::record(
        conn,
        "baseline",
        &id,
        "create",
        &serde_json::json!({ "name": name.trim(), "taskCount": task_list.len() }).to_string(),
    )?;
    get(conn, &id)
}

pub fn delete(conn: &Connection, id: &str) -> Result<()> {
    let affected = conn.execute("DELETE FROM baselines WHERE id = ?1", params![id])?;
    if affected == 0 {
        return Err(not_found(id));
    }
    oplog::record(conn, "baseline", id, "delete", "{}")
}
