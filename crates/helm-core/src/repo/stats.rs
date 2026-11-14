//! 驾驶舱统计数据。

use rusqlite::Connection;
use serde::Serialize;

use crate::error::Result;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DashboardStats {
    pub project_count: i64,
    pub active_projects: i64,
    pub tasks_todo: i64,
    pub tasks_in_progress: i64,
    pub tasks_done: i64,
    pub overdue_tasks: i64,
    pub due_soon_tasks: i64,
}

fn count(conn: &Connection, sql: &str) -> Result<i64> {
    Ok(conn.query_row(sql, [], |r| r.get(0))?)
}

pub fn get(conn: &Connection) -> Result<DashboardStats> {
    Ok(DashboardStats {
        project_count: count(conn, "SELECT COUNT(*) FROM projects")?,
        active_projects: count(
            conn,
            "SELECT COUNT(*) FROM projects WHERE status = 'active'",
        )?,
        tasks_todo: count(conn, "SELECT COUNT(*) FROM tasks WHERE status = 'todo'")?,
        tasks_in_progress: count(
            conn,
            "SELECT COUNT(*) FROM tasks WHERE status = 'in_progress'",
        )?,
        tasks_done: count(conn, "SELECT COUNT(*) FROM tasks WHERE status = 'done'")?,
        overdue_tasks: count(
            conn,
            "SELECT COUNT(*) FROM tasks
             WHERE status != 'done' AND end_date IS NOT NULL AND end_date < date('now')",
        )?,
        due_soon_tasks: count(
            conn,
            "SELECT COUNT(*) FROM tasks
             WHERE status != 'done' AND end_date IS NOT NULL
               AND end_date BETWEEN date('now') AND date('now', '+7 days')",
        )?,
    })
}
