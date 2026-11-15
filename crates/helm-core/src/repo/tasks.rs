//! 任务仓库：CRUD + 自动 WBS 编码 + OpLog 记录。

use std::collections::{HashMap, VecDeque};

use rusqlite::{Connection, Row, params};

use crate::error::{HelmError, Result};
use crate::models::{
    CreateTaskInput, MoveTaskInput, ReorderTaskInput, Task, TaskMoveDirection, TaskPriority,
    TaskStatus, UpdateTaskInput,
};
use crate::repo::{oplog, projects};
use crate::util;

const COLS: &str = "id, project_id, parent_id, wbs_code, title, notes, status, priority, estimate_hours, duration_days, actual_hours, start_date, end_date, assignee, sort_order, is_milestone, completed_at, created_at, updated_at";

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
        duration_days: row.get("duration_days")?,
        actual_hours: row.get("actual_hours")?,
        start_date: row.get("start_date")?,
        end_date: row.get("end_date")?,
        assignee: row.get("assignee")?,
        sort_order: row.get("sort_order")?,
        is_milestone: row.get("is_milestone")?,
        completed_at: row.get("completed_at")?,
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
        "INSERT INTO tasks (id, project_id, parent_id, wbs_code, title, notes, status, priority, estimate_hours, duration_days, actual_hours, start_date, end_date, assignee, sort_order, is_milestone, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, 'todo', ?7, ?8, ?9, 0, ?10, ?11, ?12, ?13, ?14, ?15, ?15)",
        params![
            id,
            input.project_id,
            input.parent_id,
            wbs,
            input.title.trim(),
            input.notes,
            input.priority,
            input.estimate_hours,
            input.duration_days,
            input.start_date,
            input.end_date,
            input.assignee,
            order,
            input.is_milestone,
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
    let old = get(conn, &input.id)?;
    let now = util::now_iso();
    let affected = conn.execute(
        "UPDATE tasks SET
            title = COALESCE(?1, title),
            notes = COALESCE(?2, notes),
            status = COALESCE(?3, status),
            priority = COALESCE(?4, priority),
            estimate_hours = COALESCE(?5, estimate_hours),
            duration_days = COALESCE(?6, duration_days),
            actual_hours = COALESCE(?7, actual_hours),
            start_date = COALESCE(?8, start_date),
            end_date = COALESCE(?9, end_date),
            assignee = COALESCE(?10, assignee),
            sort_order = COALESCE(?11, sort_order),
            is_milestone = COALESCE(?12, is_milestone),
            updated_at = ?13
         WHERE id = ?14",
        params![
            input.title,
            input.notes,
            input.status,
            input.priority,
            input.estimate_hours,
            input.duration_days,
            input.actual_hours,
            input.start_date,
            input.end_date,
            input.assignee,
            input.sort_order,
            input.is_milestone,
            now,
            input.id,
        ],
    )?;
    if affected == 0 {
        return Err(not_found(&input.id));
    }
    // completed_at 由状态流转显式维护（COALESCE 语义无法置空）：
    // 转入 done 记录时刻；离开 done 清空，保证 EV 曲线随状态回退正确撤销。
    let new_status = input.status.unwrap_or(old.status);
    if new_status == TaskStatus::Done && old.status != TaskStatus::Done {
        conn.execute(
            "UPDATE tasks SET completed_at = ?1 WHERE id = ?2",
            params![now, input.id],
        )?;
    } else if new_status != TaskStatus::Done && old.status == TaskStatus::Done {
        conn.execute(
            "UPDATE tasks SET completed_at = NULL WHERE id = ?1",
            params![input.id],
        )?;
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

// ---------- S2：WBS 重排与结构维护 ----------

/// 收集任务的全部后代 id（防环校验用）。
fn descendant_ids(conn: &Connection, task_id: &str) -> Result<Vec<String>> {
    let mut result = Vec::new();
    let mut stack = vec![task_id.to_string()];
    while let Some(current) = stack.pop() {
        let mut stmt = conn.prepare("SELECT id FROM tasks WHERE parent_id = ?1")?;
        let children: Vec<String> = stmt
            .query_map(params![current], |r| r.get::<_, String>(0))?
            .collect::<rusqlite::Result<_>>()?;
        drop(stmt);
        for child in children {
            stack.push(child.clone());
            result.push(child);
        }
    }
    Ok(result)
}

/// 全项目 WBS 重编号：按 sort_order → created_at 的兄弟顺序逐层编号。
/// 结构变化（移动/重排）后调用，保证编码始终与树形一致。
fn renumber_wbs(conn: &Connection, project_id: &str) -> Result<()> {
    let all = list_by_project(conn, project_id)?;
    // 以 "" 作为根层键
    let mut children: HashMap<String, Vec<&Task>> = HashMap::new();
    for t in &all {
        children
            .entry(t.parent_id.clone().unwrap_or_default())
            .or_default()
            .push(t);
    }
    for list in children.values_mut() {
        list.sort_by(|a, b| {
            (a.sort_order, a.created_at.as_str()).cmp(&(b.sort_order, b.created_at.as_str()))
        });
    }

    let mut updates: Vec<(String, String)> = Vec::new();
    let mut queue: VecDeque<(&Task, String)> = VecDeque::new();
    let roots = children.get("").cloned().unwrap_or_default();
    for (idx, t) in roots.iter().enumerate() {
        queue.push_back((t, (idx + 1).to_string()));
    }
    while let Some((t, code)) = queue.pop_front() {
        updates.push((t.id.clone(), code.clone()));
        if let Some(kids) = children.get(&t.id) {
            for (idx, kid) in kids.iter().enumerate() {
                queue.push_back((kid, format!("{code}.{}", idx + 1)));
            }
        }
    }

    let tx = conn.unchecked_transaction()?;
    for (id, code) in &updates {
        tx.execute(
            "UPDATE tasks SET wbs_code = ?1 WHERE id = ?2",
            params![code, id],
        )?;
    }
    tx.commit()?;
    Ok(())
}

/// WBS 重排：移动任务到新父级（`None` = 顶层末位）。
pub fn move_task(conn: &Connection, input: &MoveTaskInput) -> Result<Task> {
    let task = get(conn, &input.task_id)?;
    let project_id = task.project_id.clone();
    if let Some(new_parent) = &input.new_parent_id {
        if new_parent == &task.id {
            return Err(HelmError::Validation("任务不能移动到自身之下".into()));
        }
        let parent = get(conn, new_parent)?;
        if parent.project_id != project_id {
            return Err(HelmError::Validation("父任务必须属于同一个项目".into()));
        }
        if descendant_ids(conn, &task.id)?
            .iter()
            .any(|d| d == new_parent)
        {
            return Err(HelmError::Validation(
                "不能把任务移动到自己的子任务之下".into(),
            ));
        }
    }

    let order = count_children(conn, &project_id, input.new_parent_id.as_deref())?;
    conn.execute(
        "UPDATE tasks SET parent_id = ?1, sort_order = ?2, updated_at = ?3 WHERE id = ?4",
        params![input.new_parent_id, order, util::now_iso(), task.id],
    )?;
    oplog::record(
        conn,
        "task",
        &task.id,
        "move",
        &serde_json::to_string(input)?,
    )?;
    renumber_wbs(conn, &project_id)?;
    get(conn, &task.id)
}

/// 同级排序：上移 / 下移一位。
pub fn reorder_task(conn: &Connection, input: &ReorderTaskInput) -> Result<Task> {
    let task = get(conn, &input.task_id)?;
    let all = list_by_project(conn, &task.project_id)?;
    let siblings: Vec<&Task> = all
        .iter()
        .filter(|t| t.parent_id == task.parent_id)
        .collect();
    let idx = siblings
        .iter()
        .position(|t| t.id == task.id)
        .ok_or_else(|| HelmError::Validation("任务排序状态异常".into()))?;
    let target_idx = match input.direction {
        TaskMoveDirection::Up if idx > 0 => Some(idx - 1),
        TaskMoveDirection::Down if idx + 1 < siblings.len() => Some(idx + 1),
        _ => None,
    };
    let Some(target_idx) = target_idx else {
        return Ok(task); // 已在边界，无需移动
    };
    let target = siblings[target_idx].clone();

    let now = util::now_iso();
    let tx = conn.unchecked_transaction()?;
    tx.execute(
        "UPDATE tasks SET sort_order = ?1, updated_at = ?2 WHERE id = ?3",
        params![target.sort_order, now, task.id],
    )?;
    tx.execute(
        "UPDATE tasks SET sort_order = ?1, updated_at = ?2 WHERE id = ?3",
        params![task.sort_order, now, target.id],
    )?;
    tx.commit()?;
    oplog::record(
        conn,
        "task",
        &task.id,
        "reorder",
        &serde_json::to_string(input)?,
    )?;
    renumber_wbs(conn, &task.project_id)?;
    get(conn, &task.id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{CreateProjectInput, CreateTaskInput};
    use crate::repo::projects;

    fn setup() -> (Connection, String) {
        let conn = Connection::open_in_memory().unwrap();
        crate::migrations::run(&conn).unwrap();
        let project = projects::create(
            &conn,
            &CreateProjectInput {
                name: "测试项目".into(),
                code: None,
                method: Default::default(),
                description: None,
                start_date: None,
                target_end_date: None,
                bac: 0.0,
            },
        )
        .unwrap();
        (conn, project.id)
    }

    fn mk_task(conn: &Connection, project_id: &str, title: &str) -> String {
        create(
            conn,
            &CreateTaskInput {
                project_id: project_id.into(),
                parent_id: None,
                title: title.into(),
                notes: None,
                priority: Default::default(),
                estimate_hours: 8.0,
                duration_days: 1.0,
                start_date: None,
                end_date: None,
                assignee: None,
                is_milestone: false,
            },
        )
        .unwrap()
        .id
    }

    #[test]
    fn completing_task_records_completed_at() {
        let (conn, pid) = setup();
        let id = mk_task(&conn, &pid, "写代码");
        let done = update(
            &conn,
            &UpdateTaskInput {
                id: id.clone(),
                title: None,
                notes: None,
                status: Some(TaskStatus::Done),
                priority: None,
                estimate_hours: None,
                duration_days: None,
                actual_hours: None,
                start_date: None,
                end_date: None,
                assignee: None,
                sort_order: None,
                is_milestone: None,
            },
        )
        .unwrap();
        assert_eq!(done.status, TaskStatus::Done);
        assert!(done.completed_at.is_some(), "转入 done 应记录完成时刻");
    }

    #[test]
    fn reopening_task_clears_completed_at() {
        let (conn, pid) = setup();
        let id = mk_task(&conn, &pid, "写代码");
        update(
            &conn,
            &UpdateTaskInput {
                id: id.clone(),
                title: None,
                notes: None,
                status: Some(TaskStatus::Done),
                priority: None,
                estimate_hours: None,
                duration_days: None,
                actual_hours: None,
                start_date: None,
                end_date: None,
                assignee: None,
                sort_order: None,
                is_milestone: None,
            },
        )
        .unwrap();
        let reopened = update(
            &conn,
            &UpdateTaskInput {
                id: id.clone(),
                title: None,
                notes: None,
                status: Some(TaskStatus::InProgress),
                priority: None,
                estimate_hours: None,
                duration_days: None,
                actual_hours: None,
                start_date: None,
                end_date: None,
                assignee: None,
                sort_order: None,
                is_milestone: None,
            },
        )
        .unwrap();
        assert!(
            reopened.completed_at.is_none(),
            "离开 done 应清空完成时刻，保证 EV 曲线正确回退"
        );
    }

    #[test]
    fn unrelated_update_keeps_completed_at() {
        let (conn, pid) = setup();
        let id = mk_task(&conn, &pid, "写代码");
        update(
            &conn,
            &UpdateTaskInput {
                id: id.clone(),
                title: None,
                notes: None,
                status: Some(TaskStatus::Done),
                priority: None,
                estimate_hours: None,
                duration_days: None,
                actual_hours: None,
                start_date: None,
                end_date: None,
                assignee: None,
                sort_order: None,
                is_milestone: None,
            },
        )
        .unwrap();
        // 仅改标题：完成时刻不应被触碰
        let t = update(
            &conn,
            &UpdateTaskInput {
                id: id.clone(),
                title: Some("改个名字".into()),
                notes: None,
                status: None,
                priority: None,
                estimate_hours: None,
                duration_days: None,
                actual_hours: None,
                start_date: None,
                end_date: None,
                assignee: None,
                sort_order: None,
                is_milestone: None,
            },
        )
        .unwrap();
        assert_eq!(t.title, "改个名字");
        assert!(t.completed_at.is_some());
    }
}
