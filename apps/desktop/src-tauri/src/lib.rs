//! Tauri 壳层：只做状态装配与命令注册，业务逻辑全部在 helm-core / helm-ai。

mod ai_commands;
mod commands;
mod data_commands;

use std::sync::Mutex;

use helm_core::db::Db;
use tauri::{
    Manager, WindowEvent,
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
};

pub struct AppState {
    pub db: Mutex<Db>,
}

/// 关闭按钮是否驻留托盘（settings 表 `close_to_tray`，默认开启）。
/// 关闭拦截实时读取，设置页切换后立即生效。
fn close_to_tray_enabled(app: &tauri::AppHandle) -> bool {
    let state = app.state::<AppState>();
    let Ok(db) = state.db.lock() else {
        return true; // 锁异常时保守处理：驻留（可从托盘退出，不丢数据）
    };
    helm_core::repo::settings::get(&db.conn, "close_to_tray")
        .ok()
        .flatten()
        .map(|v| v != "false")
        .unwrap_or(true)
}

fn show_main_window(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
}

pub fn run() {
    tauri::Builder::default()
        // 单实例：必须最先注册。二次启动立即退出并把已有窗口带到前台
        // （窗口托盘驻留时同样能恢复，避免用户误以为"点不开"）。
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            show_main_window(app);
        }))
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            // 数据文件位置：%APPDATA%/com.prohelm.desktop/（Win）、~/Library/Application Support（mac）
            let dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&dir)?;
            let db = Db::open(&dir.join("prohelm.db"))?;
            app.manage(AppState { db: Mutex::new(db) });

            // 系统托盘：左键恢复窗口；菜单 = 显示主窗口 / 立即备份 / 退出
            let show_item = MenuItem::with_id(app, "show", "显示主窗口", true, None::<&str>)?;
            let backup_item = MenuItem::with_id(app, "backup", "立即备份", true, None::<&str>)?;
            let quit_item = MenuItem::with_id(app, "quit", "退出 ProHelm", true, None::<&str>)?;
            let tray_menu = Menu::with_items(app, &[&show_item, &backup_item, &quit_item])?;
            TrayIconBuilder::new()
                // Windows 托盘显示 16~32px：用 32 源缩放比 128 源更清晰
                .icon(tauri::include_image!("icons/32x32.png"))
                .tooltip("ProHelm（项舵）")
                .menu(&tray_menu)
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "show" => show_main_window(app),
                    "backup" => {
                        if let Err(e) =
                            data_commands::create_backup(app.clone(), app.state::<AppState>())
                        {
                            eprintln!("[prohelm] 托盘备份失败: {}", e.message);
                        }
                    }
                    "quit" => app.exit(0),
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        show_main_window(tray.app_handle());
                    }
                })
                .build(app)?;

            // 关闭按钮 → 驻留托盘（隐藏窗口；由托盘菜单"退出"真正退出）
            let handle = app.handle().clone();
            let closeable = app.get_webview_window("main").ok_or("主窗口不存在")?;
            let hidden_target = closeable.clone();
            closeable.on_window_event(move |event| {
                if let WindowEvent::CloseRequested { api, .. } = event
                    && close_to_tray_enabled(&handle)
                {
                    api.prevent_close();
                    let _ = hidden_target.hide();
                }
            });

            // 每日自动快照：启动后稍候执行首轮，其后每小时检查一次（当天已备份则跳过）
            let handle = app.handle().clone();
            std::thread::spawn(move || {
                std::thread::sleep(std::time::Duration::from_secs(5));
                loop {
                    data_commands::run_auto_backup_cycle(&handle);
                    std::thread::sleep(std::time::Duration::from_secs(3600));
                }
            });
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
            commands::get_telemetry,
            commands::get_evm,
            commands::create_time_log,
            commands::list_time_logs,
            commands::delete_time_log,
            data_commands::get_auto_backup_enabled,
            data_commands::set_auto_backup_enabled,
            data_commands::get_close_to_tray,
            data_commands::set_close_to_tray,
        ])
        .run(tauri::generate_context!())
        .expect("ProHelm 启动失败");
}
