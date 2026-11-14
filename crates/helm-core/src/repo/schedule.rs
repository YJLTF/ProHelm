//! 排程装配：CPM 结果 + 锚点日期 → 甘特图/排程视图数据。
//!
//! 约定（S2）：日期 = 锚点日 + 日历天偏移（暂不含工作日历，S3 引入资源日历后升级）；
//! end_date 为"含当天"（工期 3 天 → 起 D0、止 D2）。

use std::collections::HashMap;

use chrono::NaiveDate;
use rusqlite::Connection;
use serde::Serialize;

use crate::cpm;
use crate::error::{HelmError, Result};
use crate::repo::{dependencies, projects, tasks};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScheduleTask {
    pub task_id: String,
    pub wbs_code: Option<String>,
    pub title: String,
    /// 含当天（YYYY-MM-DD）
    pub start_date: String,
    /// 含当天（YYYY-MM-DD）
    pub end_date: String,
    pub es_days: f64,
    pub ef_days: f64,
    pub total_float_days: f64,
    pub critical: bool,
    pub is_milestone: bool,
    pub predecessor_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectSchedule {
    pub project_id: String,
    pub anchor_date: String,
    pub duration_days: f64,
    pub tasks: Vec<ScheduleTask>,
}

fn format_date(d: NaiveDate) -> String {
    d.format("%Y-%m-%d").to_string()
}

pub fn get(conn: &Connection, project_id: &str) -> Result<ProjectSchedule> {
    let project = projects::get(conn, project_id)?;
    let task_list = tasks::list_by_project(conn, project_id)?;
    let dep_list = dependencies::list_by_project(conn, project_id)?;

    let cpm_tasks: Vec<cpm::CpmTask> = task_list
        .iter()
        .map(|t| cpm::CpmTask {
            id: t.id.clone(),
            duration_days: t.duration_days,
        })
        .collect();
    let cpm_edges: Vec<cpm::CpmEdge> = dep_list
        .iter()
        .map(|d| cpm::CpmEdge {
            predecessor_id: d.predecessor_id.clone(),
            successor_id: d.successor_id.clone(),
            dep_type: d.dep_type,
            lag_days: d.lag_days,
        })
        .collect();
    let result = cpm::compute(&cpm_tasks, &cpm_edges)?;

    let anchor = match &project.start_date {
        Some(s) => NaiveDate::parse_from_str(s, "%Y-%m-%d")
            .map_err(|_| HelmError::Validation(format!("项目开始日期格式无效: {s}")))?,
        None => chrono::Local::now().date_naive(),
    };

    let mut predecessors: HashMap<String, Vec<String>> = HashMap::new();
    for d in &dep_list {
        predecessors
            .entry(d.successor_id.clone())
            .or_default()
            .push(d.predecessor_id.clone());
    }

    let schedule_tasks = task_list
        .iter()
        .map(|t| {
            let r = result.tasks.get(&t.id).cloned().unwrap_or_default();
            let start_offset = r.es.round() as i64;
            let end_offset = (r.ef.round() as i64 - 1).max(start_offset);
            ScheduleTask {
                task_id: t.id.clone(),
                wbs_code: t.wbs_code.clone(),
                title: t.title.clone(),
                start_date: format_date(anchor + chrono::Duration::days(start_offset)),
                end_date: format_date(anchor + chrono::Duration::days(end_offset)),
                es_days: r.es,
                ef_days: r.ef,
                total_float_days: r.total_float,
                critical: r.critical,
                is_milestone: t.is_milestone,
                predecessor_ids: predecessors.remove(&t.id).unwrap_or_default(),
            }
        })
        .collect();

    Ok(ProjectSchedule {
        project_id: project.id,
        anchor_date: format_date(anchor),
        duration_days: result.project_duration_days,
        tasks: schedule_tasks,
    })
}
