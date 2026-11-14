//! 类型化 Tauri 命令：薄封装 helm-core 仓库层，统一错误为可序列化结构。

use helm_core::repo::{self, baselines, dependencies, schedule, stats::DashboardStats};
use helm_core::{
    Baseline, CreateDependencyInput, CreateProjectInput, CreateTaskInput, Dependency,
    MoveTaskInput, Project, ProjectSchedule, ReorderTaskInput, Task, UpdateDependencyInput,
    UpdateProjectInput, UpdateTaskInput,
};
use serde::Serialize;
use tauri::State;

use crate::AppState;

/// 前端可读的错误结构（S1 用 message，S4 扩展 code/字段级信息）。
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandError {
    pub message: String,
}

impl From<helm_core::HelmError> for CommandError {
    fn from(e: helm_core::HelmError) -> Self {
        Self {
            message: e.to_string(),
        }
    }
}

type CmdResult<T> = Result<T, CommandError>;

fn with_db<T>(
    state: &State<AppState>,
    f: impl FnOnce(&helm_core::db::Db) -> helm_core::Result<T>,
) -> CmdResult<T> {
    let db = state.db.lock().map_err(|_| CommandError {
        message: "数据库连接不可用".into(),
    })?;
    Ok(f(&db)?)
}

// ---------- 项目 ----------

#[tauri::command]
pub fn list_projects(state: State<AppState>) -> CmdResult<Vec<Project>> {
    with_db(&state, |db| repo::projects::list(&db.conn))
}

#[tauri::command]
pub fn create_project(state: State<AppState>, input: CreateProjectInput) -> CmdResult<Project> {
    with_db(&state, |db| repo::projects::create(&db.conn, &input))
}

#[tauri::command]
pub fn update_project(state: State<AppState>, input: UpdateProjectInput) -> CmdResult<Project> {
    with_db(&state, |db| repo::projects::update(&db.conn, &input))
}

#[tauri::command]
pub fn delete_project(state: State<AppState>, id: String) -> CmdResult<()> {
    with_db(&state, |db| repo::projects::delete(&db.conn, &id))
}

// ---------- 任务 ----------

#[tauri::command]
pub fn list_tasks(state: State<AppState>, project_id: String) -> CmdResult<Vec<Task>> {
    with_db(&state, |db| {
        repo::tasks::list_by_project(&db.conn, &project_id)
    })
}

#[tauri::command]
pub fn create_task(state: State<AppState>, input: CreateTaskInput) -> CmdResult<Task> {
    with_db(&state, |db| repo::tasks::create(&db.conn, &input))
}

#[tauri::command]
pub fn update_task(state: State<AppState>, input: UpdateTaskInput) -> CmdResult<Task> {
    with_db(&state, |db| repo::tasks::update(&db.conn, &input))
}

#[tauri::command]
pub fn delete_task(state: State<AppState>, id: String) -> CmdResult<()> {
    with_db(&state, |db| repo::tasks::delete(&db.conn, &id))
}

// ---------- 驾驶舱 ----------

#[tauri::command]
pub fn get_dashboard_stats(state: State<AppState>) -> CmdResult<DashboardStats> {
    with_db(&state, |db| repo::stats::get(&db.conn))
}

// ---------- 进度：排程 / 依赖（S2） ----------

#[tauri::command]
pub fn get_schedule(state: State<AppState>, project_id: String) -> CmdResult<ProjectSchedule> {
    with_db(&state, |db| schedule::get(&db.conn, &project_id))
}

#[tauri::command]
pub fn create_dependency(
    state: State<AppState>,
    input: CreateDependencyInput,
) -> CmdResult<Dependency> {
    with_db(&state, |db| dependencies::create(&db.conn, &input))
}

#[tauri::command]
pub fn update_dependency(
    state: State<AppState>,
    input: UpdateDependencyInput,
) -> CmdResult<Dependency> {
    with_db(&state, |db| dependencies::update(&db.conn, &input))
}

#[tauri::command]
pub fn delete_dependency(state: State<AppState>, id: String) -> CmdResult<()> {
    with_db(&state, |db| dependencies::delete(&db.conn, &id))
}

// ---------- WBS 重排（S2） ----------

#[tauri::command]
pub fn move_task(state: State<AppState>, input: MoveTaskInput) -> CmdResult<Task> {
    with_db(&state, |db| repo::tasks::move_task(&db.conn, &input))
}

#[tauri::command]
pub fn reorder_task(state: State<AppState>, input: ReorderTaskInput) -> CmdResult<Task> {
    with_db(&state, |db| repo::tasks::reorder_task(&db.conn, &input))
}

// ---------- 基线（S2） ----------

#[tauri::command]
pub fn create_baseline(
    state: State<AppState>,
    project_id: String,
    name: String,
) -> CmdResult<Baseline> {
    with_db(&state, |db| baselines::create(&db.conn, &project_id, &name))
}

#[tauri::command]
pub fn list_baselines(state: State<AppState>, project_id: String) -> CmdResult<Vec<Baseline>> {
    with_db(&state, |db| {
        baselines::list_by_project(&db.conn, &project_id)
    })
}
