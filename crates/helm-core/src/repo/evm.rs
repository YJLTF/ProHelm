//! 挣值管理装配层：从基线快照 / 当前任务 / 工时台账汇集输入，调用 `evm::compute`。
//!
//! 基线快照是 S2 保存的完整 Task JSON，历史快照缺新字段（completedAt 等），
//! 反序列化用 `#[serde(default)]` 兜底。

use rusqlite::Connection;
use serde::Deserialize;

use crate::error::{HelmError, Result};
use crate::evm::{CurrentTask, EvmReport, LedgerDay, PlannedTask};
use crate::models::Task;
use crate::repo::{baselines, tasks, time_log};

/// 基线快照 JSON 的容器结构（与 repo::baselines::create 的序列化对齐）。
#[derive(Deserialize)]
struct BaselineSnapshot {
    #[serde(default)]
    tasks: Vec<Task>,
}

fn to_planned(all: &[Task]) -> Vec<PlannedTask> {
    all.iter()
        .map(|t| PlannedTask {
            id: t.id.clone(),
            parent_id: t.parent_id.clone(),
            estimate_hours: t.estimate_hours,
            start: t.start_date.clone(),
            end: t.end_date.clone(),
        })
        .collect()
}

fn to_current(all: &[Task]) -> Vec<CurrentTask> {
    all.iter()
        .map(|t| CurrentTask {
            id: t.id.clone(),
            parent_id: t.parent_id.clone(),
            estimate_hours: t.estimate_hours,
            done: t.status == crate::models::TaskStatus::Done,
            completed_at: t.completed_at.clone(),
        })
        .collect()
}

/// 挣值报表。`baseline_id` 缺省时取该项目最新基线。
pub fn report(conn: &Connection, project_id: &str, baseline_id: Option<&str>) -> Result<EvmReport> {
    let baseline = match baseline_id {
        Some(id) => Some(baselines::get(conn, id)?),
        None => baselines::list_by_project(conn, project_id)?
            .into_iter()
            .next(),
    };
    let Some(baseline) = baseline else {
        // 未建基线：仍给出 AC 曲线可用性信号（has_baseline=false，前端引导）
        let ledger = time_log::daily_totals(conn, project_id)?;
        return Ok(crate::evm::compute(&[], &[], &ledger, &today()));
    };

    let snap: BaselineSnapshot = serde_json::from_str(&baseline.snapshot)
        .map_err(|e| HelmError::Validation(format!("基线快照解析失败: {e}")))?;
    let current_tasks = tasks::list_by_project(conn, project_id)?;
    let ledger: Vec<LedgerDay> = time_log::daily_totals(conn, project_id)?;

    Ok(crate::evm::compute(
        &to_planned(&snap.tasks),
        &to_current(&current_tasks),
        &ledger,
        &today(),
    ))
}

/// 统计截止日（本机时区的今天）。
fn today() -> String {
    chrono::Local::now().format("%Y-%m-%d").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{CreateProjectInput, CreateTaskInput, CreateTimeLogInput, UpdateTaskInput};
    use crate::repo::{projects, time_log};

    fn setup_project(conn: &Connection, name: &str) -> String {
        projects::create(
            conn,
            &CreateProjectInput {
                name: name.into(),
                code: None,
                method: Default::default(),
                description: None,
                start_date: None,
                target_end_date: None,
                bac: 0.0,
            },
        )
        .unwrap()
        .id
    }

    fn mk_task(
        conn: &Connection,
        pid: &str,
        title: &str,
        hours: f64,
        start: &str,
        end: &str,
    ) -> String {
        tasks::create(
            conn,
            &CreateTaskInput {
                project_id: pid.into(),
                parent_id: None,
                title: title.into(),
                notes: None,
                priority: Default::default(),
                estimate_hours: hours,
                duration_days: 1.0,
                start_date: Some(start.into()),
                end_date: Some(end.into()),
                assignee: None,
                is_milestone: false,
            },
        )
        .unwrap()
        .id
    }

    #[test]
    fn no_baseline_returns_empty_report() {
        let conn = Connection::open_in_memory().unwrap();
        crate::migrations::run(&conn).unwrap();
        let pid = setup_project(&conn, "无基线");
        let report = report(&conn, &pid, None).unwrap();
        assert!(!report.has_baseline);
    }

    #[test]
    fn baseline_plus_logs_produce_full_report() {
        let conn = Connection::open_in_memory().unwrap();
        crate::migrations::run(&conn).unwrap();
        let pid = setup_project(&conn, "EVM 端到端");
        let t1 = mk_task(&conn, &pid, "需求", 10.0, "2026-03-01", "2026-03-02");
        let t2 = mk_task(&conn, &pid, "开发", 30.0, "2026-03-03", "2026-03-05");

        // 保存基线（此刻全未完成）
        baselines::create(&conn, &pid, "v1").unwrap();

        // 登记工时并完成 t1
        time_log::create(
            &conn,
            &CreateTimeLogInput {
                project_id: pid.clone(),
                task_id: t1.clone(),
                log_date: "2026-03-02".into(),
                hours: 6.0,
                note: None,
            },
        )
        .unwrap();
        time_log::create(
            &conn,
            &CreateTimeLogInput {
                project_id: pid.clone(),
                task_id: t1.clone(),
                log_date: "2026-03-02".into(),
                hours: 5.0,
                note: None,
            },
        )
        .unwrap();
        tasks::update(
            &conn,
            &UpdateTaskInput {
                id: t1.clone(),
                title: None,
                notes: None,
                status: Some(crate::models::TaskStatus::Done),
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
        let _ = t2;

        let r = report(&conn, &pid, None).unwrap();
        assert!(r.has_baseline);
        assert_eq!(r.summary.bac, 40.0);
        assert_eq!(r.summary.ev, 10.0, "完成的需求任务挣 10 人时");
        assert_eq!(r.summary.ac, 11.0);
        // 今天的日期不定，PV 取决于基线日期与今天的相对关系；只验证曲线非空且 AC 收尾
        assert!(!r.series.is_empty());
        let last = r.series.last().unwrap();
        assert_eq!(last.ac, 11.0);
        assert!(r.summary.cpi < 1.0, "11 人时完成 10 人时预算 → CPI 略低");
    }

    #[test]
    fn explicit_baseline_id_selects_that_snapshot() {
        let conn = Connection::open_in_memory().unwrap();
        crate::migrations::run(&conn).unwrap();
        let pid = setup_project(&conn, "多基线");
        let _t1 = mk_task(&conn, &pid, "任务", 8.0, "2026-03-01", "2026-03-01");
        let v1 = baselines::create(&conn, &pid, "v1").unwrap();
        // 基线后新增任务：v1 快照不含它
        let _t2 = mk_task(&conn, &pid, "基线外", 5.0, "2026-03-02", "2026-03-02");
        let v2 = baselines::create(&conn, &pid, "v2").unwrap();

        let r1 = report(&conn, &pid, Some(&v1.id)).unwrap();
        assert_eq!(r1.summary.bac, 8.0, "v1 只含一个任务");
        let r2 = report(&conn, &pid, Some(&v2.id)).unwrap();
        assert_eq!(r2.summary.bac, 13.0);
        // 默认取最新（v2）
        let r_default = report(&conn, &pid, None).unwrap();
        assert_eq!(r_default.summary.bac, 13.0);
    }
}
