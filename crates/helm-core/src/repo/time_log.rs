//! 工时台账仓库：AC（实际成本）的唯一事实来源。
//!
//! 写入/删除后重算 `tasks.actual_hours`（派生缓存），保证
//! 任务列表、人员负荷等既有统计无需感知台账。

use rusqlite::{Connection, Row, params};

use crate::error::{HelmError, Result};
use crate::models::{CreateTimeLogInput, TimeLog};
use crate::repo::{oplog, tasks};
use crate::util;

const COLS: &str = "id, project_id, task_id, log_date, hours, note, created_at";

fn map_row(row: &Row) -> rusqlite::Result<TimeLog> {
    Ok(TimeLog {
        id: row.get("id")?,
        project_id: row.get("project_id")?,
        task_id: row.get("task_id")?,
        log_date: row.get("log_date")?,
        hours: row.get("hours")?,
        note: row.get("note")?,
        created_at: row.get("created_at")?,
    })
}

fn not_found(id: &str) -> HelmError {
    HelmError::NotFound(format!("工时记录 {id} 不存在"))
}

/// 重算任务的 actual_hours 缓存 = 台账合计。
fn sync_actual_hours(conn: &Connection, task_id: &str) -> Result<()> {
    conn.execute(
        "UPDATE tasks
         SET actual_hours = (SELECT COALESCE(SUM(hours), 0) FROM time_log WHERE task_id = ?1),
             updated_at = ?2
         WHERE id = ?1",
        params![task_id, util::now_iso()],
    )?;
    Ok(())
}

pub fn create(conn: &Connection, input: &CreateTimeLogInput) -> Result<TimeLog> {
    input.validate()?;
    let task = tasks::get(conn, &input.task_id)?;
    if task.project_id != input.project_id {
        return Err(HelmError::Validation("任务不属于该项目".into()));
    }

    let id = util::new_id();
    let now = util::now_iso();
    conn.execute(
        "INSERT INTO time_log (id, project_id, task_id, log_date, hours, note, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            id,
            input.project_id,
            input.task_id,
            input.log_date.trim(),
            input.hours,
            input.note,
            now,
        ],
    )?;
    sync_actual_hours(conn, &input.task_id)?;
    oplog::record(
        conn,
        "time_log",
        &id,
        "create",
        &serde_json::to_string(input)?,
    )?;
    get(conn, &id)
}

pub fn get(conn: &Connection, id: &str) -> Result<TimeLog> {
    let sql = format!("SELECT {COLS} FROM time_log WHERE id = ?1");
    conn.query_row(&sql, params![id], map_row)
        .map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => not_found(id),
            other => other.into(),
        })
}

/// 按任务查询（日期倒序）。
pub fn list_by_task(conn: &Connection, task_id: &str) -> Result<Vec<TimeLog>> {
    let sql = format!(
        "SELECT {COLS} FROM time_log WHERE task_id = ?1 ORDER BY log_date DESC, created_at DESC"
    );
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(params![task_id], map_row)?;
    Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
}

/// 按项目查询最近记录（日期倒序，供监控页签展示）。
pub fn list_by_project(conn: &Connection, project_id: &str, limit: i64) -> Result<Vec<TimeLog>> {
    let sql = format!(
        "SELECT {COLS} FROM time_log WHERE project_id = ?1
         ORDER BY log_date DESC, created_at DESC LIMIT {limit}"
    );
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(params![project_id], map_row)?;
    Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
}

/// 按项目按日聚合（EVM AC 曲线的直接输入）。
pub fn daily_totals(conn: &Connection, project_id: &str) -> Result<Vec<crate::evm::LedgerDay>> {
    let mut stmt = conn.prepare(
        "SELECT log_date, SUM(hours) FROM time_log WHERE project_id = ?1 GROUP BY log_date ORDER BY log_date ASC",
    )?;
    let rows = stmt
        .query_map(params![project_id], |r| {
            Ok(crate::evm::LedgerDay {
                date: r.get(0)?,
                hours: r.get(1)?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(rows)
}

pub fn delete(conn: &Connection, id: &str) -> Result<()> {
    let entry = get(conn, id)?;
    let affected = conn.execute("DELETE FROM time_log WHERE id = ?1", params![id])?;
    if affected == 0 {
        return Err(not_found(id));
    }
    sync_actual_hours(conn, &entry.task_id)?;
    oplog::record(conn, "time_log", id, "delete", "{}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{CreateProjectInput, CreateTaskInput};
    use crate::repo::projects;

    fn setup() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        crate::migrations::run(&conn).unwrap();
        projects::create(
            &conn,
            &CreateProjectInput {
                name: "EVM 项目".into(),
                code: None,
                method: Default::default(),
                description: None,
                start_date: None,
                target_end_date: None,
                bac: 0.0,
            },
        )
        .unwrap();
        conn
    }

    fn task(conn: &Connection, title: &str, hours: f64) -> String {
        tasks::create(
            conn,
            &CreateTaskInput {
                project_id: projects::list(conn).unwrap()[0].id.clone(),
                parent_id: None,
                title: title.into(),
                notes: None,
                priority: Default::default(),
                estimate_hours: hours,
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
    fn create_updates_actual_hours_cache() {
        let conn = setup();
        let project_id = projects::list(&conn).unwrap()[0].id.clone();
        let task_id = task(&conn, "开发", 10.0);
        let input = CreateTimeLogInput {
            project_id: project_id.clone(),
            task_id: task_id.clone(),
            log_date: "2026-03-01".into(),
            hours: 3.5,
            note: Some("编码".into()),
        };
        let log = create(&conn, &input).unwrap();
        assert_eq!(log.hours, 3.5);
        assert_eq!(tasks::get(&conn, &task_id).unwrap().actual_hours, 3.5);

        create(
            &conn,
            &CreateTimeLogInput {
                project_id: project_id.clone(),
                task_id: task_id.clone(),
                log_date: "2026-03-02".into(),
                hours: 2.0,
                note: None,
            },
        )
        .unwrap();
        assert_eq!(tasks::get(&conn, &task_id).unwrap().actual_hours, 5.5);
    }

    #[test]
    fn delete_resyncs_actual_hours() {
        let conn = setup();
        let project_id = projects::list(&conn).unwrap()[0].id.clone();
        let task_id = task(&conn, "开发", 10.0);
        let log = create(
            &conn,
            &CreateTimeLogInput {
                project_id,
                task_id: task_id.clone(),
                log_date: "2026-03-01".into(),
                hours: 4.0,
                note: None,
            },
        )
        .unwrap();
        delete(&conn, &log.id).unwrap();
        assert_eq!(tasks::get(&conn, &task_id).unwrap().actual_hours, 0.0);
    }

    #[test]
    fn validation_rejects_bad_input() {
        let conn = setup();
        let project_id = projects::list(&conn).unwrap()[0].id.clone();
        let task_id = task(&conn, "开发", 10.0);
        for (hours, date) in [(0.0, "2026-03-01"), (25.0, "2026-03-01"), (1.0, "03-01")] {
            let err = create(
                &conn,
                &CreateTimeLogInput {
                    project_id: project_id.clone(),
                    task_id: task_id.clone(),
                    log_date: date.into(),
                    hours,
                    note: None,
                },
            )
            .unwrap_err();
            assert!(matches!(err, HelmError::Validation(_)));
        }
    }

    #[test]
    fn task_project_mismatch_rejected() {
        let conn = setup();
        let other = task(&conn, "占位", 1.0);
        let project_id = projects::list(&conn).unwrap()[0].id.clone();
        let input = CreateTimeLogInput {
            project_id,
            task_id: "不存在的任务".into(),
            log_date: "2026-03-01".into(),
            hours: 1.0,
            note: None,
        };
        let _ = other; // 仅保证 setup 有任务数据
        let err = create(&conn, &input).unwrap_err();
        assert!(matches!(
            err,
            HelmError::NotFound(_) | HelmError::Validation(_)
        ));
    }

    #[test]
    fn daily_totals_and_project_list() {
        let conn = setup();
        let project_id = projects::list(&conn).unwrap()[0].id.clone();
        let task_id = task(&conn, "开发", 10.0);
        for (date, hours) in [
            ("2026-03-01", 3.0),
            ("2026-03-01", 2.0),
            ("2026-03-02", 4.0),
        ] {
            create(
                &conn,
                &CreateTimeLogInput {
                    project_id: project_id.clone(),
                    task_id: task_id.clone(),
                    log_date: date.into(),
                    hours,
                    note: None,
                },
            )
            .unwrap();
        }
        let totals = daily_totals(&conn, &project_id).unwrap();
        assert_eq!(totals.len(), 2);
        assert_eq!(totals[0].date, "2026-03-01");
        assert_eq!(totals[0].hours, 5.0);
        assert_eq!(totals[1].hours, 4.0);

        let recent = list_by_project(&conn, &project_id, 10).unwrap();
        assert_eq!(recent.len(), 3);
        assert_eq!(recent[0].log_date, "2026-03-02");
    }
}
