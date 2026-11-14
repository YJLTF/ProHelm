//! Tauri 壳层：只做状态装配与命令注册，业务逻辑全部在 helm-core / helm-ai。

mod ai_commands;
mod commands;
mod data_commands;

use std::sync::Mutex;

use helm_core::db::Db;
use tauri::Manager;

pub struct AppState {
    pub db: Mutex<Db>,
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            // 数据文件位置：%APPDATA%/com.prohelm.desktop/（Win）、~/Library/Application Support（mac）
            let dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&dir)?;
            let db = Db::open(&dir.join("prohelm.db"))?;
            app.manage(AppState { db: Mutex::new(db) });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::list_projects,
            commands::create_project,
            commands::update_project,
            commands::delete_project,
            commands::list_tasks,
            commands::create_task,
            commands::update_task,
            commands::delete_task,
            commands::get_dashboard_stats,
            commands::get_schedule,
            commands::create_dependency,
            commands::update_dependency,
            commands::delete_dependency,
            commands::move_task,
            commands::reorder_task,
            commands::create_baseline,
            commands::list_baselines,
            commands::list_risks,
            commands::create_risk,
            commands::update_risk,
            commands::delete_risk,
            commands::list_stakeholders,
            commands::create_stakeholder,
            commands::update_stakeholder,
            commands::delete_stakeholder,
            commands::get_charter,
            commands::save_charter,
            commands::set_tailored,
            commands::apply_recommended_tailoring,
            ai_commands::get_ai_settings,
            ai_commands::set_ai_settings,
            ai_commands::test_ai_connection,
            ai_commands::coach_ask,
            ai_commands::generate_charter_draft,
            ai_commands::generate_wbs_draft,
            ai_commands::list_ai_artifacts,
            ai_commands::set_artifact_status,
            data_commands::create_backup,
            data_commands::list_backups,
            data_commands::delete_backup,
            data_commands::restore_backup,
            data_commands::export_snapshot_to,
            data_commands::import_snapshot_from,
            data_commands::export_project_report_to,
            data_commands::export_tasks_csv_to,
            commands::get_risk_radar,
            commands::get_workload,
        ])
        .run(tauri::generate_context!())
        .expect("ProHelm 启动失败");
}
