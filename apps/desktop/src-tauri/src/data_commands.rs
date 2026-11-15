//! 数据安全命令：备份/恢复、快照导入导出、项目报告与任务 CSV 导出。
//! 文件选择对话框由前端插件（@tauri-apps/plugin-dialog）完成，本层只负责读写。

use std::path::PathBuf;

use rusqlite::params;
use serde::Serialize;
use tauri::{AppHandle, Manager, State};

use helm_core::db::Db;

use crate::AppState;
use crate::commands::CommandError;

type CmdResult<T> = Result<T, CommandError>;

fn io_error(e: std::io::Error) -> CommandError {
    CommandError {
        message: format!("文件操作失败: {e}"),
    }
}

fn lock_error() -> CommandError {
    CommandError {
        message: "数据库连接不可用".into(),
    }
}

fn db_file_path(app: &AppHandle) -> CmdResult<PathBuf> {
    app.path()
        .app_data_dir()
        .map(|dir| dir.join("prohelm.db"))
        .map_err(|e| CommandError {
            message: e.to_string(),
        })
}

fn backups_dir(app: &AppHandle) -> CmdResult<PathBuf> {
    let dir = app
        .path()
        .app_data_dir()
        .map(|dir| dir.join("backups"))
        .map_err(|e| CommandError {
            message: e.to_string(),
        })?;
    std::fs::create_dir_all(&dir).map_err(io_error)?;
    Ok(dir)
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupFileInfo {
    pub file_name: String,
    pub path: String,
    pub size_bytes: u64,
    pub created_at: String,
}

fn file_info(path: &PathBuf) -> CmdResult<BackupFileInfo> {
    let meta = std::fs::metadata(path).map_err(io_error)?;
    let created_at = meta
        .modified()
        .ok()
        .map(|t| {
            chrono::DateTime::<chrono::Local>::from(t)
                .format("%Y-%m-%d %H:%M:%S")
                .to_string()
        })
        .unwrap_or_default();
    Ok(BackupFileInfo {
        file_name: path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default(),
        path: path.to_string_lossy().to_string(),
        size_bytes: meta.len(),
        created_at,
    })
}

fn validate_backup_file_name(file_name: &str) -> CmdResult<()> {
    if file_name.contains('/') || file_name.contains('\\') || file_name.contains("..") {
        return Err(CommandError {
            message: "非法的备份文件名".into(),
        });
    }
    Ok(())
}

// ---------- 每日自动快照（S6） ----------

/// 自动快照的保留数量：超出后按文件名时间淘汰最旧的 auto 备份。
const AUTO_BACKUP_KEEP: usize = 14;

/// 执行一轮自动快照检查：当天未备份过（且开关开启）则落一个 auto 备份并修剪历史。
/// 启动时与每小时由后台线程调用；任何失败静默记录，不干扰正常使用。
pub fn run_auto_backup_cycle(app: &AppHandle) {
    let result = try_auto_backup(app, &app.state::<AppState>());
    if let Err(e) = result {
        eprintln!("[prohelm] 自动快照失败: {}", e.message);
    }
}

fn try_auto_backup(app: &AppHandle, state: &State<AppState>) -> CmdResult<()> {
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    let (enabled, last_date) = {
        let db = state.db.lock().map_err(|_| lock_error())?;
        let enabled = helm_core::repo::settings::get(&db.conn, "auto_backup_enabled")?
            .map(|v| v != "false")
            .unwrap_or(true); // 默认开启：数据安全优先
        let last = helm_core::repo::settings::get(&db.conn, "last_auto_backup_date")?;
        (enabled, last)
    };
    if !enabled || last_date.as_deref() == Some(today.as_str()) {
        return Ok(());
    }

    let dir = backups_dir(app)?;
    let ts = chrono::Local::now().format("%Y%m%d-%H%M%S").to_string();
    let target = dir.join(format!("prohelm-auto-{ts}.db"));
    {
        let db = state.db.lock().map_err(|_| lock_error())?;
        db.conn
            .execute("VACUUM INTO ?1", params![target.to_string_lossy().as_ref()])
            .map_err(|e| CommandError {
                message: format!("自动备份失败: {e}"),
            })?;
        helm_core::repo::settings::set(&db.conn, "last_auto_backup_date", &today)?;
    }
    prune_auto_backups(&dir)?;
    Ok(())
}

/// 保留最近 `AUTO_BACKUP_KEEP` 个 auto 备份，删除更旧的。
fn prune_auto_backups(dir: &PathBuf) -> CmdResult<()> {
    let mut autos: Vec<PathBuf> = std::fs::read_dir(dir)
        .map_err(io_error)?
        .flatten()
        .map(|e| e.path())
        .filter(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.starts_with("prohelm-auto-") && n.ends_with(".db"))
        })
        .collect();
    if autos.len() <= AUTO_BACKUP_KEEP {
        return Ok(());
    }
    autos.sort(); // 文件名含时间戳，字典序即时间序
    let excess = autos.len() - AUTO_BACKUP_KEEP;
    for path in autos.into_iter().take(excess) {
        let _ = std::fs::remove_file(path);
    }
    Ok(())
}

/// 自动快照开关（默认开启）。
#[tauri::command]
pub fn get_auto_backup_enabled(state: State<AppState>) -> CmdResult<bool> {
    let db = state.db.lock().map_err(|_| lock_error())?;
    let v = helm_core::repo::settings::get(&db.conn, "auto_backup_enabled")?;
    Ok(v.map(|s| s != "false").unwrap_or(true))
}

#[tauri::command]
pub fn set_auto_backup_enabled(state: State<AppState>, enabled: bool) -> CmdResult<()> {
    let db = state.db.lock().map_err(|_| lock_error())?;
    helm_core::repo::settings::set(
        &db.conn,
        "auto_backup_enabled",
        if enabled { "true" } else { "false" },
    )?;
    Ok(())
}

/// 关闭按钮驻留托盘开关（默认开启；关闭拦截实时读取此设置）。
#[tauri::command]
pub fn get_close_to_tray(state: State<AppState>) -> CmdResult<bool> {
    let db = state.db.lock().map_err(|_| lock_error())?;
    let v = helm_core::repo::settings::get(&db.conn, "close_to_tray")?;
    Ok(v.map(|s| s != "false").unwrap_or(true))
}

#[tauri::command]
pub fn set_close_to_tray(state: State<AppState>, enabled: bool) -> CmdResult<()> {
    let db = state.db.lock().map_err(|_| lock_error())?;
    helm_core::repo::settings::set(
        &db.conn,
        "close_to_tray",
        if enabled { "true" } else { "false" },
    )?;
    Ok(())
}

// ---------- 备份与恢复 ----------

#[tauri::command]
pub fn create_backup(app: AppHandle, state: State<AppState>) -> CmdResult<BackupFileInfo> {
    let dir = backups_dir(&app)?;
    let ts = chrono::Local::now().format("%Y%m%d-%H%M%S").to_string();
    let mut target = dir.join(format!("prohelm-backup-{ts}.db"));
    let mut n = 1u32;
    while target.exists() {
        n += 1;
        target = dir.join(format!("prohelm-backup-{ts}-{n}.db"));
    }
    {
        let db = state.db.lock().map_err(|_| lock_error())?;
        db.conn
            .execute("VACUUM INTO ?1", params![target.to_string_lossy().as_ref()])
            .map_err(|e| CommandError {
                message: format!("备份失败: {e}"),
            })?;
    }
    file_info(&target)
}

#[tauri::command]
pub fn list_backups(app: AppHandle) -> CmdResult<Vec<BackupFileInfo>> {
    let dir = backups_dir(&app)?;
    let mut items = Vec::new();
    let entries = std::fs::read_dir(&dir).map_err(io_error)?;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some("db") {
            items.push(file_info(&path)?);
        }
    }
    items.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    Ok(items)
}

#[tauri::command]
pub fn delete_backup(app: AppHandle, file_name: String) -> CmdResult<()> {
    validate_backup_file_name(&file_name)?;
    let path = backups_dir(&app)?.join(&file_name);
    if path.exists() {
        std::fs::remove_file(&path).map_err(io_error)?;
    }
    Ok(())
}

#[tauri::command]
pub fn restore_backup(app: AppHandle, state: State<AppState>, file_name: String) -> CmdResult<()> {
    validate_backup_file_name(&file_name)?;
    let source = backups_dir(&app)?.join(&file_name);
    if !source.exists() {
        return Err(CommandError {
            message: format!("备份文件不存在: {file_name}"),
        });
    }
    // 校验 SQLite 文件头
    let header = std::fs::read(&source).map_err(io_error)?;
    if header.len() < 16 || &header[..16] != b"SQLite format 3\0" {
        return Err(CommandError {
            message: "备份文件不是有效的 SQLite 数据库".into(),
        });
    }

    // 安全兜底：恢复前自动备份当前库
    create_backup(app.clone(), state.clone())?;

    let db_path = db_file_path(&app)?;
    let mut guard = state.db.lock().map_err(|_| lock_error())?;
    // 先用内存库占位并释放文件连接，再覆盖与重开
    let placeholder = Db::open_in_memory().map_err(|e| CommandError {
        message: e.to_string(),
    })?;
    let old = std::mem::replace(&mut *guard, placeholder);
    drop(old);
    std::fs::copy(&source, &db_path).map_err(io_error)?;
    let new_db = Db::open(&db_path).map_err(|e| CommandError {
        message: e.to_string(),
    })?;
    *guard = new_db;
    Ok(())
}

// ---------- 快照导入导出 ----------

#[tauri::command]
pub fn export_snapshot_to(state: State<AppState>, target_path: String) -> CmdResult<()> {
    let json = {
        let db = state.db.lock().map_err(|_| lock_error())?;
        let snap = helm_core::snapshot::export(&db.conn)?;
        serde_json::to_string_pretty(&snap)?
    };
    std::fs::write(&target_path, json).map_err(io_error)
}

#[tauri::command]
pub fn import_snapshot_from(
    state: State<AppState>,
    source_path: String,
) -> CmdResult<helm_core::ImportStats> {
    let text = std::fs::read_to_string(&source_path).map_err(io_error)?;
    let snap: helm_core::Snapshot = serde_json::from_str(&text).map_err(|e| CommandError {
        message: format!("数据包解析失败: {e}"),
    })?;
    let db = state.db.lock().map_err(|_| lock_error())?;
    helm_core::snapshot::import(&db.conn, &snap).map_err(Into::into)
}

// ---------- 报告与 CSV 导出 ----------

#[tauri::command]
pub fn export_project_report_to(
    state: State<AppState>,
    project_id: String,
    target_path: String,
) -> CmdResult<()> {
    let md = {
        let db = state.db.lock().map_err(|_| lock_error())?;
        helm_core::report::project_report_markdown(&db.conn, &project_id)?
    };
    std::fs::write(&target_path, md).map_err(io_error)
}

#[tauri::command]
pub fn export_tasks_csv_to(
    state: State<AppState>,
    project_id: String,
    target_path: String,
) -> CmdResult<()> {
    let csv = {
        let db = state.db.lock().map_err(|_| lock_error())?;
        helm_core::report::tasks_csv(&db.conn, &project_id)?
    };
    std::fs::write(&target_path, csv).map_err(io_error)
}
