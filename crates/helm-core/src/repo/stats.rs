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

// ---------- 内测诊断统计（S6，opt-in 数据埋点的本地优先实现） ----------

/// 全部指标从 OpLog / 业务表本地聚合，不含任何内容文本，无网络上报。
/// 内测用户可在设置页查看并主动复制这段匿名 JSON 反馈给团队。
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TelemetrySummary {
    /// 首次使用时间（最早 OpLog 时间戳）。
    pub first_seen_at: Option<String>,
    /// 最近一次写操作时间。
    pub last_active_at: Option<String>,
    /// 累计有写操作的天数。
    pub active_days: i64,
    /// 最近 7 天活跃天数（周留存的本地代理指标）。
    pub active_days_last7: i64,
    pub projects_created: i64,
    pub tasks_created: i64,
    pub risks_created: i64,
    pub baselines_created: i64,
    pub time_logs_created: i64,
    /// AI 生成总次数（教练问答 / 章程 / WBS 草稿）。
    pub ai_generations: i64,
    pub ai_adopted: i64,
    /// M1 激活标准：创建 ≥1 个项目且 ≥5 个任务。
    pub activated: bool,
}

pub fn telemetry(conn: &Connection) -> Result<TelemetrySummary> {
    let (first_seen_at, last_active_at, active_days, active_days_last7): (
        Option<String>,
        Option<String>,
        i64,
        i64,
    ) = conn.query_row(
        "SELECT MIN(ts), MAX(ts),
                COUNT(DISTINCT substr(ts, 1, 10)),
                COUNT(DISTINCT CASE WHEN substr(ts, 1, 10) >= date('now', '-6 days')
                                    THEN substr(ts, 1, 10) END)
         FROM oplog",
        [],
        |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
    )?;

    let (projects_created, tasks_created, risks_created, baselines_created, time_logs_created): (
        i64,
        i64,
        i64,
        i64,
        i64,
    ) = conn.query_row(
        "SELECT
            COALESCE(SUM(entity = 'project' AND op = 'create'), 0),
            COALESCE(SUM(entity = 'task' AND op = 'create'), 0),
            COALESCE(SUM(entity = 'risk' AND op = 'create'), 0),
            COALESCE(SUM(entity = 'baseline' AND op = 'create'), 0),
            COALESCE(SUM(entity = 'time_log' AND op = 'create'), 0)
         FROM oplog",
        [],
        |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?)),
    )?;

    let (ai_generations, ai_adopted): (i64, i64) = conn.query_row(
        "SELECT COUNT(*), COALESCE(SUM(status = 'adopted'), 0) FROM ai_artifacts",
        [],
        |r| Ok((r.get(0)?, r.get(1)?)),
    )?;

    Ok(TelemetrySummary {
        first_seen_at,
        last_active_at,
        active_days,
        active_days_last7,
        projects_created,
        tasks_created,
        risks_created,
        baselines_created,
        time_logs_created,
        ai_generations,
        ai_adopted,
        activated: projects_created >= 1 && tasks_created >= 5,
    })
}

#[cfg(test)]
mod telemetry_tests {
    use super::*;
    use crate::models::{AiArtifactKind, CreateProjectInput, CreateTaskInput, CreateTimeLogInput};
    use crate::repo::{ai_artifacts, projects, tasks, time_log};

    fn setup() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        crate::migrations::run(&conn).unwrap();
        conn
    }

    #[test]
    fn empty_database_reports_zeros() {
        let conn = setup();
        let t = telemetry(&conn).unwrap();
        assert!(!t.activated);
        assert_eq!(t.projects_created, 0);
        assert!(t.first_seen_at.is_none());
        assert_eq!(t.active_days, 0);
    }

    #[test]
    fn aggregates_oplog_and_ai_artifacts() {
        let conn = setup();
        let project = projects::create(
            &conn,
            &CreateProjectInput {
                name: "遥测".into(),
                code: None,
                method: Default::default(),
                description: None,
                start_date: None,
                target_end_date: None,
                bac: 0.0,
            },
        )
        .unwrap();
        for i in 0..5 {
            tasks::create(
                &conn,
                &CreateTaskInput {
                    project_id: project.id.clone(),
                    parent_id: None,
                    title: format!("任务{i}"),
                    notes: None,
                    priority: Default::default(),
                    estimate_hours: 1.0,
                    duration_days: 1.0,
                    start_date: None,
                    end_date: None,
                    assignee: None,
                    is_milestone: false,
                },
            )
            .unwrap();
        }
        let task_id = tasks::list_by_project(&conn, &project.id).unwrap()[0]
            .id
            .clone();
        time_log::create(
            &conn,
            &CreateTimeLogInput {
                project_id: project.id.clone(),
                task_id,
                log_date: "2026-03-01".into(),
                hours: 1.0,
                note: None,
            },
        )
        .unwrap();
        ai_artifacts::create(
            &conn,
            AiArtifactKind::Coach,
            None,
            "test-provider",
            "test-model",
            "问",
            "答",
        )
        .unwrap();

        let t = telemetry(&conn).unwrap();
        assert_eq!(t.projects_created, 1);
        assert_eq!(t.tasks_created, 5);
        assert_eq!(t.time_logs_created, 1);
        assert_eq!(t.ai_generations, 1);
        assert_eq!(t.ai_adopted, 0, "草稿未采纳不计入采纳数");
        assert!(t.activated, "1 项目 + 5 任务应判定激活");
        assert!(t.first_seen_at.is_some());
        assert!(t.active_days >= 1);
        assert!(t.active_days_last7 >= 1, "刚写入的数据落在最近 7 天");
    }
}
