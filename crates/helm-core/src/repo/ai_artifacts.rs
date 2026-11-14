//! AI 生成溯源仓库：记录 / 检索 / 状态流转。

use rusqlite::{Connection, Row, params};

use crate::error::{HelmError, Result};
use crate::models::{AiArtifact, AiArtifactKind, AiArtifactStatus};
use crate::util;

const COLS: &str =
    "id, project_id, kind, provider, model, input_summary, output, status, created_at";

fn map_row(row: &Row) -> rusqlite::Result<AiArtifact> {
    let kind: String = row.get("kind")?;
    let status: String = row.get("status")?;
    Ok(AiArtifact {
        id: row.get("id")?,
        project_id: row.get("project_id")?,
        kind: AiArtifactKind::parse(&kind).map_err(crate::repo::enum_conversion_error)?,
        provider: row.get("provider")?,
        model: row.get("model")?,
        input_summary: row.get("input_summary")?,
        output: row.get("output")?,
        status: AiArtifactStatus::parse(&status).map_err(crate::repo::enum_conversion_error)?,
        created_at: row.get("created_at")?,
    })
}

fn not_found(id: &str) -> HelmError {
    HelmError::NotFound(format!("AI 记录 {id} 不存在"))
}

/// 记录一次 AI 生成。`input_summary` 截断到 200 字符。
pub fn create(
    conn: &Connection,
    kind: AiArtifactKind,
    project_id: Option<&str>,
    provider: &str,
    model: &str,
    input_summary: &str,
    output: &str,
) -> Result<AiArtifact> {
    let id = util::new_id();
    let summary: String = input_summary.chars().take(200).collect();
    conn.execute(
        "INSERT INTO ai_artifacts (id, project_id, kind, provider, model, input_summary, output, status, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 'draft', ?8)",
        params![id, project_id, kind, provider, model, summary, output, util::now_iso()],
    )?;
    get(conn, &id)
}

/// 按创建时间倒序；`project_id` 为 Some 时过滤该项目，None 返回全部。
pub fn list(conn: &Connection, project_id: Option<&str>) -> Result<Vec<AiArtifact>> {
    match project_id {
        Some(pid) => {
            let sql = format!(
                "SELECT {COLS} FROM ai_artifacts WHERE project_id = ?1 ORDER BY created_at DESC"
            );
            let mut stmt = conn.prepare(&sql)?;
            let rows = stmt.query_map(params![pid], map_row)?;
            Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
        }
        None => {
            let sql = format!("SELECT {COLS} FROM ai_artifacts ORDER BY created_at DESC");
            let mut stmt = conn.prepare(&sql)?;
            let rows = stmt.query_map([], map_row)?;
            Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
        }
    }
}

pub fn get(conn: &Connection, id: &str) -> Result<AiArtifact> {
    let sql = format!("SELECT {COLS} FROM ai_artifacts WHERE id = ?1");
    conn.query_row(&sql, params![id], map_row)
        .map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => not_found(id),
            other => other.into(),
        })
}

pub fn set_status(conn: &Connection, id: &str, status: AiArtifactStatus) -> Result<AiArtifact> {
    let affected = conn.execute(
        "UPDATE ai_artifacts SET status = ?1 WHERE id = ?2",
        params![status, id],
    )?;
    if affected == 0 {
        return Err(not_found(id));
    }
    get(conn, id)
}
