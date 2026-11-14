//! 风险登记册仓库：CRUD + 项目内自增编号 + OpLog 记录。

use rusqlite::{Connection, Row, params};

use crate::error::{HelmError, Result};
use crate::models::{
    CreateRiskInput, ResponseStrategy, Risk, RiskCategory, RiskStatus, UpdateRiskInput,
};
use crate::repo::{oplog, projects};
use crate::util;

const COLS: &str = "id, project_id, risk_no, title, detail, category, probability, impact, response_strategy, response_plan, owner, trigger_desc, status, created_at, updated_at";

fn map_row(row: &Row) -> rusqlite::Result<Risk> {
    let category: String = row.get("category")?;
    let strategy: String = row.get("response_strategy")?;
    let status: String = row.get("status")?;
    Ok(Risk {
        id: row.get("id")?,
        project_id: row.get("project_id")?,
        risk_no: row.get("risk_no")?,
        title: row.get("title")?,
        detail: row.get("detail")?,
        category: RiskCategory::parse(&category).map_err(crate::repo::enum_conversion_error)?,
        probability: row.get("probability")?,
        impact: row.get("impact")?,
        response_strategy: ResponseStrategy::parse(&strategy)
            .map_err(crate::repo::enum_conversion_error)?,
        response_plan: row.get("response_plan")?,
        owner: row.get("owner")?,
        trigger_desc: row.get("trigger_desc")?,
        status: RiskStatus::parse(&status).map_err(crate::repo::enum_conversion_error)?,
        created_at: row.get("created_at")?,
        updated_at: row.get("updated_at")?,
    })
}

fn not_found(id: &str) -> HelmError {
    HelmError::NotFound(format!("风险 {id} 不存在"))
}

/// 按评分（概率×影响）降序、编号升序。
pub fn list_by_project(conn: &Connection, project_id: &str) -> Result<Vec<Risk>> {
    let sql = format!(
        "SELECT {COLS} FROM risks WHERE project_id = ?1
         ORDER BY probability * impact DESC, risk_no ASC"
    );
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(params![project_id], map_row)?;
    Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
}

pub fn get(conn: &Connection, id: &str) -> Result<Risk> {
    let sql = format!("SELECT {COLS} FROM risks WHERE id = ?1");
    conn.query_row(&sql, params![id], map_row)
        .map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => not_found(id),
            other => other.into(),
        })
}

pub fn create(conn: &Connection, input: &CreateRiskInput) -> Result<Risk> {
    input.validate()?;
    projects::get(conn, &input.project_id)?;

    let risk_no: i64 = conn.query_row(
        "SELECT COALESCE(MAX(risk_no), 0) + 1 FROM risks WHERE project_id = ?1",
        params![input.project_id],
        |r| r.get(0),
    )?;
    let id = util::new_id();
    conn.execute(
        "INSERT INTO risks (id, project_id, risk_no, title, detail, category, probability, impact, response_strategy, response_plan, owner, trigger_desc, status, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, 'open', ?13, ?13)",
        params![
            id,
            input.project_id,
            risk_no,
            input.title.trim(),
            input.detail,
            input.category,
            input.probability,
            input.impact,
            input.response_strategy,
            input.response_plan,
            input.owner,
            input.trigger_desc,
            util::now_iso(),
        ],
    )?;
    oplog::record(conn, "risk", &id, "create", &serde_json::to_string(input)?)?;
    get(conn, &id)
}

pub fn update(conn: &Connection, input: &UpdateRiskInput) -> Result<Risk> {
    input.validate()?;
    let affected = conn.execute(
        "UPDATE risks SET
            title = COALESCE(?1, title),
            detail = COALESCE(?2, detail),
            category = COALESCE(?3, category),
            probability = COALESCE(?4, probability),
            impact = COALESCE(?5, impact),
            response_strategy = COALESCE(?6, response_strategy),
            response_plan = COALESCE(?7, response_plan),
            owner = COALESCE(?8, owner),
            trigger_desc = COALESCE(?9, trigger_desc),
            status = COALESCE(?10, status),
            updated_at = ?11
         WHERE id = ?12",
        params![
            input.title,
            input.detail,
            input.category,
            input.probability,
            input.impact,
            input.response_strategy,
            input.response_plan,
            input.owner,
            input.trigger_desc,
            input.status,
            util::now_iso(),
            input.id,
        ],
    )?;
    if affected == 0 {
        return Err(not_found(&input.id));
    }
    oplog::record(
        conn,
        "risk",
        &input.id,
        "update",
        &serde_json::to_string(input)?,
    )?;
    get(conn, &input.id)
}

pub fn delete(conn: &Connection, id: &str) -> Result<()> {
    let affected = conn.execute("DELETE FROM risks WHERE id = ?1", params![id])?;
    if affected == 0 {
        return Err(not_found(id));
    }
    oplog::record(conn, "risk", id, "delete", "{}")
}
