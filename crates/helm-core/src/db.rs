use std::path::Path;

use rusqlite::Connection;

use crate::{error::Result, migrations};

/// 数据库句柄。桌面端全局单例，由上层用 `Mutex<Db>` 包装以支持多线程命令调用。
pub struct Db {
    pub conn: Connection,
}

impl Db {
    /// 打开（或创建）数据库文件并执行迁移。
    pub fn open(path: &Path) -> Result<Db> {
        let conn = Connection::open(path)?;
        Self::init(conn)
    }

    /// 测试与集成场景使用。
    pub fn open_in_memory() -> Result<Db> {
        let conn = Connection::open_in_memory()?;
        Self::init(conn)
    }

    fn init(conn: Connection) -> Result<Db> {
        conn.execute_batch(
            "PRAGMA foreign_keys = ON;
             PRAGMA journal_mode = WAL;",
        )?;
        migrations::run(&conn)?;
        Ok(Db { conn })
    }
}
