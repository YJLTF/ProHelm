//! 任务依赖仓库：CRUD + 环检测 + OpLog 记录。

use std::collections::{HashMap, VecDeque};

use rusqlite::{Connection, Row, params};

use crate::error::{HelmError, Result};
use crate::models::{CreateDependencyInput, Dependency, DependencyType, UpdateDependencyInput};
use crate::repo::{oplog, tasks};
use crate::util;

const COLS: &str = "id, project_id, predecessor_id, successor_id, dep_type, lag_days, created_at";

fn map_row(row: &Row) -> rusqlite::Result<Dependency> {
    let dep_type: String = row.get("dep_type")?;
    Ok(Dependency {
        id: row.get("id")?,
        project_id: row.get("project_id")?,
        predecessor_id: row.get("predecessor_id")?,
        successor_id: row.get("successor_id")?,
        dep_type: DependencyType::parse(&dep_type).map_err(crate::repo::enum_conversion_error)?,
        lag_days: row.get("lag_days")?,
        created_at: row.get("created_at")?,
    })
}

fn not_found(id: &str) -> HelmError {
    HelmError::NotFound(format!("依赖 {id} 不存在"))
}

pub fn list_by_project(conn: &Connection, project_id: &str) -> Result<Vec<Dependency>> {
    let sql =
        format!("SELECT {COLS} FROM dependencies WHERE project_id = ?1 ORDER BY created_at ASC");
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(params![project_id], map_row)?;
    Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
}

pub fn get(conn: &Connection, id: &str) -> Result<Dependency> {
    let sql = format!("SELECT {COLS} FROM dependencies WHERE id = ?1");
    conn.query_row(&sql, params![id], map_row)
        .map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => not_found(id),
            other => other.into(),
        })
}

/// 在现有依赖 + 一条候选边上做 Kahn 拓扑排序；返回 true 表示会形成环。
fn would_create_cycle(
    conn: &Connection,
    project_id: &str,
    new_pred: &str,
    new_succ: &str,
) -> Result<bool> {
    let task_list = tasks::list_by_project(conn, project_id)?;
    let dep_list = list_by_project(conn, project_id)?;

    let mut index: HashMap<&str, usize> = HashMap::new();
    for (i, t) in task_list.iter().enumerate() {
        index.insert(t.id.as_str(), i);
    }
    let n = task_list.len();
    let mut adjacency: Vec<Vec<usize>> = vec![Vec::new(); n];
    let mut indegree: Vec<usize> = vec![0; n];

    fn add_edge(
        index: &HashMap<&str, usize>,
        adjacency: &mut [Vec<usize>],
        indegree: &mut [usize],
        pred: &str,
        succ: &str,
    ) -> Result<()> {
        let p = index
            .get(pred)
            .ok_or_else(|| HelmError::Validation(format!("依赖的前置任务不存在: {pred}")))?;
        let s = index
            .get(succ)
            .ok_or_else(|| HelmError::Validation(format!("依赖的后继任务不存在: {succ}")))?;
        adjacency[*p].push(*s);
        indegree[*s] += 1;
        Ok(())
    }

    for d in &dep_list {
        add_edge(
            &index,
            &mut adjacency,
            &mut indegree,
            &d.predecessor_id,
            &d.successor_id,
        )?;
    }
    add_edge(&index, &mut adjacency, &mut indegree, new_pred, new_succ)?;

    let mut ready: VecDeque<usize> = (0..n).filter(|&i| indegree[i] == 0).collect();
    let mut processed = 0usize;
    while let Some(i) = ready.pop_front() {
        processed += 1;
        for &s in &adjacency[i] {
            indegree[s] -= 1;
            if indegree[s] == 0 {
                ready.push_back(s);
            }
        }
    }
    Ok(processed < n)
}

pub fn create(conn: &Connection, input: &CreateDependencyInput) -> Result<Dependency> {
    if input.predecessor_id == input.successor_id {
        return Err(HelmError::Validation("任务不能依赖自身".into()));
    }
    if !input.lag_days.is_finite() {
        return Err(HelmError::Validation("滞后天数必须为有限数值".into()));
    }
    let pred = tasks::get(conn, &input.predecessor_id)?;
    let succ = tasks::get(conn, &input.successor_id)?;
    if pred.project_id != input.project_id || succ.project_id != input.project_id {
        return Err(HelmError::Validation(
            "依赖的两个任务必须属于同一个项目".into(),
        ));
    }

    let duplicate: i64 = conn.query_row(
        "SELECT COUNT(*) FROM dependencies WHERE predecessor_id = ?1 AND successor_id = ?2",
        params![input.predecessor_id, input.successor_id],
        |r| r.get(0),
    )?;
    if duplicate > 0 {
        return Err(HelmError::Validation("已存在相同的依赖关系".into()));
    }
    if would_create_cycle(
        conn,
        &input.project_id,
        &input.predecessor_id,
        &input.successor_id,
    )? {
        return Err(HelmError::Validation(
            "新增依赖会形成循环引用，请检查任务间的先后关系".into(),
        ));
    }

    let id = util::new_id();
    conn.execute(
        "INSERT INTO dependencies (id, project_id, predecessor_id, successor_id, dep_type, lag_days, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            id,
            input.project_id,
            input.predecessor_id,
            input.successor_id,
            input.dep_type,
            input.lag_days,
            util::now_iso(),
        ],
    )?;
    oplog::record(
        conn,
        "dependency",
        &id,
        "create",
        &serde_json::to_string(input)?,
    )?;
    get(conn, &id)
}

pub fn update(conn: &Connection, input: &UpdateDependencyInput) -> Result<Dependency> {
    if input.lag_days.is_some_and(|v| !v.is_finite()) {
        return Err(HelmError::Validation("滞后天数必须为有限数值".into()));
    }
    let affected = conn.execute(
        "UPDATE dependencies SET
            dep_type = COALESCE(?1, dep_type),
            lag_days = COALESCE(?2, lag_days)
         WHERE id = ?3",
        params![input.dep_type, input.lag_days, input.id],
    )?;
    if affected == 0 {
        return Err(not_found(&input.id));
    }
    oplog::record(
        conn,
        "dependency",
        &input.id,
        "update",
        &serde_json::to_string(input)?,
    )?;
    get(conn, &input.id)
}

pub fn delete(conn: &Connection, id: &str) -> Result<()> {
    let affected = conn.execute("DELETE FROM dependencies WHERE id = ?1", params![id])?;
    if affected == 0 {
        return Err(not_found(id));
    }
    oplog::record(conn, "dependency", id, "delete", "{}")
}
