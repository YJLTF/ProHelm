//! 任务仓库：CRUD + 自动 WBS 编码 + OpLog 记录。

use rusqlite::{Connection, Row, params};

use crate::error::{HelmError, Result};
use crate::models::{CreateTaskInput, Task, TaskPriority, TaskStatus, UpdateTaskInput};
use crate::repo::{oplog, projects};
use crate::util;

const COLS: &str = "id, project_id, parent_id, wbs_code, title, notes, status, priority, estimate_hours, actual_hours, start_date, end_date, assignee, sort_order, created_at, updated_at";

fn map_row(row: &Row) -> rusqlite::Result<Task> {
    let status: String = row.get("status")?;
    let priority: String = row.get("priority")?;
    Ok(Task {
        id: row.get("id")?,
        project_id: row.get("project_id")?,
        parent_id: row.get("parent_id")?,
        wbs_code: row.get("wbs_code")?,
        title: row.get("title")?,
        notes: row.get("notes")?,
        status: TaskStatus::parse(&status).map_err(crate::repo::enum_conversion_error)?,
        priority: TaskPriority::parse(&priority).map_err(crate::repo::enum_conversion_error)?,
        estimate_hours: row.get("estimate_hours")?,
        actual_hours: row.get("actual_hours")?,
        start_date: row.get("start_date")?,
        end_date: row.get("end_date")?,
        assignee: row.get("assignee")?,
        sort_order: row.get("sort_order")?,
        created_at: row.get("created_at")?,
        updated_at: row.get("updated_at")?,
    })
}

fn not_found(id: &str) -> HelmError {
    HelmError::NotFound(format!("任务 {id} 不存在"))
}

fn count_children(conn: &Connection, project_id: &str, parent_id: Option<&str>) -> Result<i64> {
    conn.query_row(
        "SELECT COUNT(*) FROM tasks WHERE project_id = ?1 AND parent_id IS ?2",
        params![project_id, parent_id],
        |r| r.get(0),
    )
    .map_err(Into::into)
}

/// 自动维护 WBS 编码：顶层为 1、2、3…，子任务为 父编码.序号。
fn next_wbs_code(conn: &Connection, project_id: &str, parent_id: Option<&str>) -> Result<String> {
    let index = count_children(conn, project_id, parent_id)? + 1;
    match parent_id {
        None => Ok(format!("{index}")),
        Some(pid) => {
            let parent = get(conn, pid)?;
            let base = parent.wbs_code.clone().unwrap_or_default();
            Ok(format!("{base}.{index}"))
        }
    }
}

pub fn list_by_project(conn: &Connection, project_id: &str) -> Result<Vec<Task>> {
    let sql = format!(
        "SELECT {COLS} FROM tasks WHERE project_id = ?1 ORDER BY sort_order ASC, created_at ASC"
    );
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(params![project_id], map_row)?;
    Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
}

pub fn get(conn: &Connection, id: &str) -> Result<Task> {
    let sql = format!("SELECT {COLS} FROM tasks WHERE id = ?1");
    conn.query_row(&sql, params![id], map_row)
        .map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => not_found(id),
            other => other.into(),
        })
}

pub fn create(conn: &Connection, input: &CreateTaskInput) -> Result<Task> {
    input.validate()?;
    // 父任务必须存在且属于同一项目。
    if let Some(parent_id) = &input.parent_id {
        let parent = get(conn, parent_id)?;
        if parent.project_id != input.project_id {
            return Err(HelmError::Validation("父任务必须属于同一个项目".into()));
        }
    }
    let project = projects::get(conn, &input.project_id)?;

    let id = util::new_id();
    let now = util::now_iso();
    let order = count_children(conn, &input.project_id, input.parent_id.as_deref())?;
    let wbs = next_wbs_code(conn, &input.project_id, input.parent_id.as_deref())?;
    conn.execute(
        "INSERT INTO tasks (id, project_id, parent_id, wbs_code, title, notes, status, priority, estimate_hours, actual_hours, start_date, end_date, assignee, sort_order, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, 'todo', ?7, ?8, 0, ?9, ?10, ?11, ?12, ?13, ?13)",
        params![
            id,
            input.project_id,
            input.parent_id,
            wbs,
            input.title.trim(),
            input.notes,
            input.priority,
            input.estimate_hours,
            input.start_date,
            input.end_date,
            input.assignee,
            order,
            now,
        ],
    )?;
    // 项目有进展即视为激活（S3 裁剪器落地后由流程驱动）。
    if project.status == crate::models::ProjectStatus::Planning {
        projects::update(
            conn,
            &crate::models::UpdateProjectInput {
                id: project.id.clone(),
                name: None,
                code: None,
                method: None,
                status: Some(crate::models::ProjectStatus::Active),
                description: None,
                start_date: None,
                target_end_date: None,
                bac: None,
            },
        )?;
    }
    oplog::record(conn, "task", &id, "create", &serde_json::to_string(input)?)?;
    get(conn, &id)
}

pub fn update(conn: &Connection, input: &UpdateTaskInput) -> Result<Task> {
    input.validate()?;
    let now = util::now_iso();
    let affected = conn.execute(
        "UPDATE tasks SET
            title = COALESCE(?1, title),
            notes = COALESCE(?2, notes),
            status = COALESCE(?3, status),
            priority = COALESCE(?4, priority),
            estimate_hours = COALESCE(?5, estimate_hours),
            actual_hours = COALESCE(?6, actual_hours),
            start_date = COALESCE(?7, start_date),
            end_date = COALESCE(?8, end_date),
            assignee = COALESCE(?9, assignee),
            sort_order = COALESCE(?10, sort_order),
            updated_at = ?11
         WHERE id = ?12",
        params![
            input.title,
            input.notes,
            input.status,
            input.priority,
            input.estimate_hours,
            input.actual_hours,
            input.start_date,
            input.end_date,
            input.assignee,
            input.sort_order,
            now,
            input.id,
        ],
    )?;
    if affected == 0 {
        return Err(not_found(&input.id));
    }
    oplog::record(
        conn,
        "task",
        &input.id,
        "update",
        &serde_json::to_string(input)?,
    )?;
    get(conn, &input.id)
}

pub fn delete(conn: &Connection, id: &str) -> Result<()> {
    let affected = conn.execute("DELETE FROM tasks WHERE id = ?1", params![id])?;
    if affected == 0 {
        return Err(not_found(id));
    }
    oplog::record(conn, "task", id, "delete", "{}")
}
