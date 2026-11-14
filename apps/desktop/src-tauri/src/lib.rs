//! Tauri 壳层：只做状态装配与命令注册，业务逻辑全部在 helm-core。

mod commands;

use std::sync::Mutex;

use helm_core::db::Db;
use tauri::Manager;

pub struct AppState {
    pub db: Mutex<Db>,
}

pub fn run() {
    tauri::Builder::default()
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
        ])
        .run(tauri::generate_context!())
        .expect("ProHelm 启动失败");
}
