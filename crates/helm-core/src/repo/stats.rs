//! 驾驶舱统计数据。

use rusqlite::{Connection, Row, params};
use serde::Serialize;

use crate::error::Result;
use crate::models::RiskStatus;

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

/// 跨项目风险雷达：全部未关闭风险按评分降序。
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RiskRadarItem {
    pub risk_id: String,
    pub project_id: String,
    pub project_name: String,
    pub risk_no: i64,
    pub title: String,
    pub probability: i64,
    pub impact: i64,
    pub owner: Option<String>,
    pub status: RiskStatus,
}

impl RiskRadarItem {
    pub fn score(&self) -> i64 {
        self.probability * self.impact
    }
}

pub fn risk_radar(conn: &Connection, limit: i64) -> Result<Vec<RiskRadarItem>> {
    let sql = "SELECT r.id, r.project_id, p.name AS project_name, r.risk_no, r.title,
                      r.probability, r.impact, r.owner, r.status
               FROM risks r JOIN projects p ON p.id = r.project_id
               WHERE r.status != 'closed'
               ORDER BY r.probability * r.impact DESC, r.risk_no ASC
               LIMIT ?1";
    let mut stmt = conn.prepare(sql)?;
    let map = |row: &Row| -> rusqlite::Result<RiskRadarItem> {
        let status: String = row.get("status")?;
        Ok(RiskRadarItem {
            risk_id: row.get("id")?,
            project_id: row.get("project_id")?,
            project_name: row.get("project_name")?,
            risk_no: row.get("risk_no")?,
            title: row.get("title")?,
            probability: row.get("probability")?,
            impact: row.get("impact")?,
            owner: row.get("owner")?,
            status: RiskStatus::parse(&status).map_err(crate::repo::enum_conversion_error)?,
        })
    };
    let rows = stmt.query_map(params![limit], map)?;
    Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
}

/// 人员负荷（按任务负责人文本聚合，成员表 S5 后升级为外键）。
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkloadItem {
    pub assignee: String,
    pub project_count: i64,
    pub task_count: i64,
    pub estimate_hours: f64,
    pub overdue: i64,
}

pub fn workload(conn: &Connection, limit: i64) -> Result<Vec<WorkloadItem>> {
    let sql = "SELECT assignee,
                      COUNT(DISTINCT project_id) AS project_count,
                      COUNT(*) AS task_count,
                      COALESCE(SUM(estimate_hours), 0) AS estimate_hours,
                      COALESCE(SUM(CASE WHEN status != 'done' AND end_date IS NOT NULL AND end_date < date('now') THEN 1 ELSE 0 END), 0) AS overdue
               FROM tasks
               WHERE assignee IS NOT NULL AND assignee != ''
               GROUP BY assignee
               ORDER BY task_count DESC
               LIMIT ?1";
    let mut stmt = conn.prepare(sql)?;
    let rows = stmt.query_map(params![limit], |row| {
        Ok(WorkloadItem {
            assignee: row.get("assignee")?,
            project_count: row.get("project_count")?,
            task_count: row.get("task_count")?,
            estimate_hours: row.get("estimate_hours")?,
            overdue: row.get("overdue")?,
        })
    })?;
    Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
}
