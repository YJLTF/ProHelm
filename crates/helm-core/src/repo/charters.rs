//! 项目章程仓库：分节 JSON 存储，读取时懒初始化默认空章程。

use rusqlite::{Connection, Row, params};

use crate::error::{HelmError, Result};
use crate::models::{CharterContent, CharterEnvelope};
use crate::repo::{oplog, projects};
use crate::util;

const COLS: &str = "id, project_id, content, updated_at";

fn parse_content(json: &str) -> Result<CharterContent> {
    serde_json::from_str(json)
        .map_err(|e| HelmError::Validation(format!("章程数据损坏，无法解析: {e}")))
}

fn envelope(row: &Row) -> rusqlite::Result<(String, String, String, String)> {
    Ok((
        row.get("id")?,
        row.get("project_id")?,
        row.get("content")?,
        row.get("updated_at")?,
    ))
}

pub fn get_or_create(conn: &Connection, project_id: &str) -> Result<CharterEnvelope> {
    projects::get(conn, project_id)?; // 项目存在性校验（404 语义）
    let sql = format!("SELECT {COLS} FROM charters WHERE project_id = ?1");
    match conn.query_row(&sql, params![project_id], envelope) {
        Ok((id, pid, content, updated_at)) => Ok(CharterEnvelope {
            id,
            project_id: pid,
            charter: parse_content(&content)?,
            updated_at,
            created: false,
        }),
        Err(rusqlite::Error::QueryReturnedNoRows) => {
            let default = CharterContent::default();
            save(conn, project_id, &default)?;
            let mut env = get_or_create(conn, project_id)?;
            env.created = true;
            Ok(env)
        }
        Err(e) => Err(e.into()),
    }
}

pub fn save(
    conn: &Connection,
    project_id: &str,
    content: &CharterContent,
) -> Result<CharterEnvelope> {
    projects::get(conn, project_id)?;
    let id = util::new_id();
    let now = util::now_iso();
    let json = serde_json::to_string(content)?;
    conn.execute(
        "INSERT INTO charters (id, project_id, content, updated_at) VALUES (?1, ?2, ?3, ?4)
         ON CONFLICT(project_id) DO UPDATE SET content = excluded.content, updated_at = excluded.updated_at",
        params![id, project_id, json, now],
    )?;
    oplog::record(conn, "charter", project_id, "save", &json)?;
    let sql = format!("SELECT {COLS} FROM charters WHERE project_id = ?1");
    let (id, pid, content, updated_at) = conn
        .query_row(&sql, params![project_id], envelope)
        .map_err(Into::<HelmError>::into)?;
    Ok(CharterEnvelope {
        id,
        project_id: pid,
        charter: parse_content(&content)?,
        updated_at,
        created: false,
    })
}
