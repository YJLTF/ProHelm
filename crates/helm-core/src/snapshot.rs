//! 全量数据快照：JSON 导出/导入（数据主权与迁移能力的基础）。
//!
//! 快照包含用户数据（项目/任务/依赖/风险/干系人/章程/基线/工时台账），
//! 不包含：oplog（本地审计）、ai_artifacts（本地溯源）、settings（本机配置）。
//! 导入保留原始 ULID 主键；先清空 projects（FK 级联清空全部子表）再插入。

use rusqlite::{Connection, Transaction, params};
use serde::{Deserialize, Serialize};

use crate::error::{HelmError, Result};
use crate::models::{
    Baseline, CharterContent, Dependency, DependencyType, Project, ProjectMethod, ProjectStatus,
    ResponseStrategy, Risk, RiskCategory, RiskStatus, Stakeholder, Task, TaskPriority, TaskStatus,
    TimeLog,
};

pub const SNAPSHOT_VERSION: i64 = 1;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snapshot {
    pub app: String,
    pub version: i64,
    pub exported_at: String,
    pub projects: Vec<Project>,
    pub tasks: Vec<Task>,
    pub dependencies: Vec<Dependency>,
    pub risks: Vec<Risk>,
    pub stakeholders: Vec<Stakeholder>,
    pub charters: Vec<SnapshotCharter>,
    pub baselines: Vec<Baseline>,
    /// S6 起随包携带工时台账；`default` 兼容旧版数据包。
    #[serde(default)]
    pub time_logs: Vec<TimeLog>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SnapshotCharter {
    pub id: String,
    pub project_id: String,
    pub content: CharterContent,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportStats {
    pub projects: usize,
    pub tasks: usize,
    pub dependencies: usize,
    pub risks: usize,
    pub stakeholders: usize,
    pub charters: usize,
    pub baselines: usize,
    pub time_logs: usize,
}

fn enum_conversion_error(e: HelmError) -> rusqlite::Error {
    crate::repo::enum_conversion_error(e)
}

// ---------- 导出 ----------

pub fn export(conn: &Connection) -> Result<Snapshot> {
    let mut projects = Vec::new();
    {
        let mut stmt = conn.prepare(
            "SELECT id, name, code, method, status, description, start_date, target_end_date, bac, tailored, created_at, updated_at FROM projects ORDER BY created_at",
        )?;
        let rows = stmt.query_map([], |row| {
            let method: String = row.get("method")?;
            let status: String = row.get("status")?;
            let tailored_raw: Option<String> = row.get("tailored")?;
            let tailored = match tailored_raw {
                Some(s) => {
                    let parsed = serde_json::from_str::<Vec<String>>(&s)
                        .map_err(|e| HelmError::Validation(format!("裁剪配置损坏: {e}")))
                        .map_err(enum_conversion_error)?;
                    Some(parsed)
                }
                None => None,
            };
            Ok(Project {
                id: row.get("id")?,
                name: row.get("name")?,
                code: row.get("code")?,
                method: ProjectMethod::parse(&method).map_err(enum_conversion_error)?,
                status: ProjectStatus::parse(&status).map_err(enum_conversion_error)?,
                description: row.get("description")?,
                start_date: row.get("start_date")?,
                target_end_date: row.get("target_end_date")?,
                bac: row.get("bac")?,
                tailored,
                created_at: row.get("created_at")?,
                updated_at: row.get("updated_at")?,
            })
        })?;
        for r in rows {
            projects.push(r?);
        }
    }

    let mut tasks = Vec::new();
    {
        let mut stmt = conn.prepare(
            "SELECT id, project_id, parent_id, wbs_code, title, notes, status, priority, estimate_hours, duration_days, actual_hours, start_date, end_date, assignee, sort_order, is_milestone, completed_at, created_at, updated_at FROM tasks ORDER BY created_at",
        )?;
        let rows = stmt.query_map([], |row| {
            let status: String = row.get("status")?;
            let priority: String = row.get("priority")?;
            Ok(Task {
                id: row.get("id")?,
                project_id: row.get("project_id")?,
                parent_id: row.get("parent_id")?,
                wbs_code: row.get("wbs_code")?,
                title: row.get("title")?,
                notes: row.get("notes")?,
                status: TaskStatus::parse(&status).map_err(enum_conversion_error)?,
                priority: TaskPriority::parse(&priority).map_err(enum_conversion_error)?,
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
        })?;
        for r in rows {
            tasks.push(r?);
        }
    }

    let mut dependencies = Vec::new();
    {
        let mut stmt = conn.prepare(
            "SELECT id, project_id, predecessor_id, successor_id, dep_type, lag_days, created_at FROM dependencies ORDER BY created_at",
        )?;
        let rows = stmt.query_map([], |row| {
            let dep_type: String = row.get("dep_type")?;
            Ok(Dependency {
                id: row.get("id")?,
                project_id: row.get("project_id")?,
                predecessor_id: row.get("predecessor_id")?,
                successor_id: row.get("successor_id")?,
                dep_type: DependencyType::parse(&dep_type).map_err(enum_conversion_error)?,
                lag_days: row.get("lag_days")?,
                created_at: row.get("created_at")?,
            })
        })?;
        for r in rows {
            dependencies.push(r?);
        }
    }

    let mut risks = Vec::new();
    {
        let mut stmt = conn.prepare(
            "SELECT id, project_id, risk_no, title, detail, category, probability, impact, response_strategy, response_plan, owner, trigger_desc, status, created_at, updated_at FROM risks ORDER BY created_at",
        )?;
        let rows = stmt.query_map([], |row| {
            let category: String = row.get("category")?;
            let strategy: String = row.get("response_strategy")?;
            let status: String = row.get("status")?;
            Ok(Risk {
                id: row.get("id")?,
                project_id: row.get("project_id")?,
                risk_no: row.get("risk_no")?,
                title: row.get("title")?,
                detail: row.get("detail")?,
                category: RiskCategory::parse(&category).map_err(enum_conversion_error)?,
                probability: row.get("probability")?,
                impact: row.get("impact")?,
                response_strategy: ResponseStrategy::parse(&strategy)
                    .map_err(enum_conversion_error)?,
                response_plan: row.get("response_plan")?,
                owner: row.get("owner")?,
                trigger_desc: row.get("trigger_desc")?,
                status: RiskStatus::parse(&status).map_err(enum_conversion_error)?,
                created_at: row.get("created_at")?,
                updated_at: row.get("updated_at")?,
            })
        })?;
        for r in rows {
            risks.push(r?);
        }
    }

    let mut stakeholders = Vec::new();
    {
        let mut stmt = conn.prepare(
            "SELECT id, project_id, name, role, expectation, influence, interest, notes, created_at, updated_at FROM stakeholders ORDER BY created_at",
        )?;
        let rows = stmt.query_map([], |row| {
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
        })?;
        for r in rows {
            stakeholders.push(r?);
        }
    }

    let mut charters = Vec::new();
    {
        let mut stmt = conn
            .prepare("SELECT id, project_id, content, updated_at FROM charters ORDER BY rowid")?;
        let rows = stmt.query_map([], |row| {
            let content: String = row.get("content")?;
            Ok(SnapshotCharter {
                id: row.get("id")?,
                project_id: row.get("project_id")?,
                content: serde_json::from_str(&content).map_err(|e| {
                    enum_conversion_error(HelmError::Validation(format!("章程数据损坏: {e}")))
                })?,
                updated_at: row.get("updated_at")?,
            })
        })?;
        for r in rows {
            charters.push(r?);
        }
    }

    let mut baselines = Vec::new();
    {
        let mut stmt = conn.prepare(
            "SELECT id, project_id, name, snapshot, created_at FROM baselines ORDER BY created_at",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(Baseline {
                id: row.get("id")?,
                project_id: row.get("project_id")?,
                name: row.get("name")?,
                snapshot: row.get("snapshot")?,
                created_at: row.get("created_at")?,
            })
        })?;
        for r in rows {
            baselines.push(r?);
        }
    }

    let mut time_logs = Vec::new();
    {
        let mut stmt = conn.prepare(
            "SELECT id, project_id, task_id, log_date, hours, note, created_at FROM time_log ORDER BY created_at",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(TimeLog {
                id: row.get("id")?,
                project_id: row.get("project_id")?,
                task_id: row.get("task_id")?,
                log_date: row.get("log_date")?,
                hours: row.get("hours")?,
                note: row.get("note")?,
                created_at: row.get("created_at")?,
            })
        })?;
        for r in rows {
            time_logs.push(r?);
        }
    }

    Ok(Snapshot {
        app: "prohelm".into(),
        version: SNAPSHOT_VERSION,
        exported_at: crate::util::now_iso(),
        projects,
        tasks,
        dependencies,
        risks,
        stakeholders,
        charters,
        baselines,
        time_logs,
    })
}

// ---------- 导入 ----------

pub fn import(conn: &Connection, snapshot: &Snapshot) -> Result<ImportStats> {
    if snapshot.app != "prohelm" || snapshot.version != SNAPSHOT_VERSION {
        return Err(HelmError::Validation(
            "不是有效的 ProHelm 数据包（app/version 不匹配）".into(),
        ));
    }

    // 章程/裁剪的 JSON 字段先行序列化（失败则整体失败，不动数据库）
    let mut tailored_json: Vec<Option<String>> = Vec::with_capacity(snapshot.projects.len());
    for p in &snapshot.projects {
        tailored_json.push(match &p.tailored {
            Some(keys) => Some(serde_json::to_string(keys)?),
            None => None,
        });
    }
    let mut charter_json: Vec<String> = Vec::with_capacity(snapshot.charters.len());
    for c in &snapshot.charters {
        charter_json.push(serde_json::to_string(&c.content)?);
    }

    // 自引用（tasks.parent_id）在批量导入期间临时关闭外键校验
    conn.execute_batch("PRAGMA foreign_keys = OFF;")?;
    let tx = conn.unchecked_transaction()?;
    let stats = match insert_all(&tx, snapshot, &tailored_json, &charter_json) {
        Ok(stats) => stats,
        Err(e) => {
            let _ = tx.rollback();
            let _ = conn.execute_batch("PRAGMA foreign_keys = ON;");
            return Err(e);
        }
    };
    tx.commit()?;
    conn.execute_batch("PRAGMA foreign_keys = ON;")?;
    Ok(stats)
}

fn insert_all(
    tx: &Transaction<'_>,
    snapshot: &Snapshot,
    tailored_json: &[Option<String>],
    charter_json: &[String],
) -> Result<ImportStats> {
    // 外键校验已临时关闭：显式按子表优先清空（旧数据 + 级联依赖）
    tx.execute("DELETE FROM ai_artifacts", [])?;
    tx.execute("DELETE FROM time_log", [])?;
    tx.execute("DELETE FROM tasks", [])?;
    tx.execute("DELETE FROM dependencies", [])?;
    tx.execute("DELETE FROM risks", [])?;
    tx.execute("DELETE FROM stakeholders", [])?;
    tx.execute("DELETE FROM charters", [])?;
    tx.execute("DELETE FROM baselines", [])?;
    tx.execute("DELETE FROM projects", [])?;
    tx.execute("DELETE FROM oplog", [])?;

    for (p, tj) in snapshot.projects.iter().zip(tailored_json.iter()) {
        tx.execute(
            "INSERT INTO projects (id, name, code, method, status, description, start_date, target_end_date, bac, tailored, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            params![
                p.id, p.name, p.code, p.method.as_str(), p.status.as_str(), p.description,
                p.start_date, p.target_end_date, p.bac, tj, p.created_at, p.updated_at
            ],
        )?;
    }
    for t in &snapshot.tasks {
        tx.execute(
            "INSERT INTO tasks (id, project_id, parent_id, wbs_code, title, notes, status, priority, estimate_hours, duration_days, actual_hours, start_date, end_date, assignee, sort_order, is_milestone, completed_at, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19)",
            params![
                t.id, t.project_id, t.parent_id, t.wbs_code, t.title, t.notes,
                t.status.as_str(), t.priority.as_str(), t.estimate_hours, t.duration_days,
                t.actual_hours, t.start_date, t.end_date, t.assignee, t.sort_order,
                t.is_milestone, t.completed_at, t.created_at, t.updated_at
            ],
        )?;
    }
    for d in &snapshot.dependencies {
        tx.execute(
            "INSERT INTO dependencies (id, project_id, predecessor_id, successor_id, dep_type, lag_days, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![d.id, d.project_id, d.predecessor_id, d.successor_id, d.dep_type.as_str(), d.lag_days, d.created_at],
        )?;
    }
    for r in &snapshot.risks {
        tx.execute(
            "INSERT INTO risks (id, project_id, risk_no, title, detail, category, probability, impact, response_strategy, response_plan, owner, trigger_desc, status, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)",
            params![
                r.id, r.project_id, r.risk_no, r.title, r.detail, r.category.as_str(),
                r.probability, r.impact, r.response_strategy.as_str(), r.response_plan,
                r.owner, r.trigger_desc, r.status.as_str(), r.created_at, r.updated_at
            ],
        )?;
    }
    for s in &snapshot.stakeholders {
        tx.execute(
            "INSERT INTO stakeholders (id, project_id, name, role, expectation, influence, interest, notes, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                s.id, s.project_id, s.name, s.role, s.expectation, s.influence, s.interest,
                s.notes, s.created_at, s.updated_at
            ],
        )?;
    }
    for (c, cj) in snapshot.charters.iter().zip(charter_json.iter()) {
        tx.execute(
            "INSERT INTO charters (id, project_id, content, updated_at) VALUES (?1, ?2, ?3, ?4)",
            params![c.id, c.project_id, cj, c.updated_at],
        )?;
    }
    for b in &snapshot.baselines {
        tx.execute(
            "INSERT INTO baselines (id, project_id, name, snapshot, created_at) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![b.id, b.project_id, b.name, b.snapshot, b.created_at],
        )?;
    }
    for l in &snapshot.time_logs {
        tx.execute(
            "INSERT INTO time_log (id, project_id, task_id, log_date, hours, note, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![l.id, l.project_id, l.task_id, l.log_date, l.hours, l.note, l.created_at],
        )?;
    }

    Ok(ImportStats {
        projects: snapshot.projects.len(),
        tasks: snapshot.tasks.len(),
        dependencies: snapshot.dependencies.len(),
        risks: snapshot.risks.len(),
        stakeholders: snapshot.stakeholders.len(),
        charters: snapshot.charters.len(),
        baselines: snapshot.baselines.len(),
        time_logs: snapshot.time_logs.len(),
    })
}
