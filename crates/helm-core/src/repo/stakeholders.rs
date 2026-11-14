//! 干系人登记册仓库：CRUD + 权力/利益方格推导 + OpLog 记录。

use rusqlite::{Connection, Row, params};

use crate::error::{HelmError, Result};
use crate::models::{CreateStakeholderInput, Stakeholder, UpdateStakeholderInput};
use crate::repo::{oplog, projects};
use crate::util;

const COLS: &str =
    "id, project_id, name, role, expectation, influence, interest, notes, created_at, updated_at";

fn map_row(row: &Row) -> rusqlite::Result<Stakeholder> {
    Ok(Stakeholder {
        id: row.get("id")?,
        project_id: row.get("project_id")?,
        name: row.get("name")?,
        role: row.get("role")?,
        expectation: row.get("expectation")?,
        influence: row.get("influence")?,
        interest: row.get("interest")?,
        notes: row.get("notes")?,
        created_at: row.get("created_at")?,
        updated_at: row.get("updated_at")?,
    })
}

fn not_found(id: &str) -> HelmError {
    HelmError::NotFound(format!("干系人 {id} 不存在"))
}

pub fn list_by_project(conn: &Connection, project_id: &str) -> Result<Vec<Stakeholder>> {
    let sql = format!(
        "SELECT {COLS} FROM stakeholders WHERE project_id = ?1 ORDER BY influence DESC, interest DESC, created_at ASC"
    );
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(params![project_id], map_row)?;
    Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
}

pub fn get(conn: &Connection, id: &str) -> Result<Stakeholder> {
    let sql = format!("SELECT {COLS} FROM stakeholders WHERE id = ?1");
    conn.query_row(&sql, params![id], map_row)
        .map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => not_found(id),
            other => other.into(),
        })
}

pub fn create(conn: &Connection, input: &CreateStakeholderInput) -> Result<Stakeholder> {
    input.validate()?;
    projects::get(conn, &input.project_id)?;
    let id = util::new_id();
    conn.execute(
        "INSERT INTO stakeholders (id, project_id, name, role, expectation, influence, interest, notes, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?9)",
        params![
            id,
            input.project_id,
            input.name.trim(),
            input.role,
            input.expectation,
            input.influence,
            input.interest,
            input.notes,
            util::now_iso(),
        ],
    )?;
    oplog::record(
        conn,
        "stakeholder",
        &id,
        "create",
        &serde_json::to_string(input)?,
    )?;
    get(conn, &id)
}

pub fn update(conn: &Connection, input: &UpdateStakeholderInput) -> Result<Stakeholder> {
    input.validate()?;
    let affected = conn.execute(
        "UPDATE stakeholders SET
            name = COALESCE(?1, name),
            role = COALESCE(?2, role),
            expectation = COALESCE(?3, expectation),
            influence = COALESCE(?4, influence),
            interest = COALESCE(?5, interest),
            notes = COALESCE(?6, notes),
            updated_at = ?7
         WHERE id = ?8",
        params![
            input.name,
            input.role,
            input.expectation,
            input.influence,
            input.interest,
            input.notes,
            util::now_iso(),
            input.id,
        ],
    )?;
    if affected == 0 {
        return Err(not_found(&input.id));
    }
    oplog::record(
        conn,
        "stakeholder",
        &input.id,
        "update",
        &serde_json::to_string(input)?,
    )?;
    get(conn, &input.id)
}

pub fn delete(conn: &Connection, id: &str) -> Result<()> {
    let affected = conn.execute("DELETE FROM stakeholders WHERE id = ?1", params![id])?;
    if affected == 0 {
        return Err(not_found(id));
    }
    oplog::record(conn, "stakeholder", id, "delete", "{}")
}
