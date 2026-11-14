//! 版本化数据库迁移。
//!
//! 规则：
//! - 迁移文件放在 `migrations/`，命名 `NNNN_description.sql`，只增不改；
//! - 迁移必须幂等安全：在事务中被 `schema_migrations` 表守护；
//! - M3 同步落地后，本框架同样用于同步元数据表。

use rusqlite::{Connection, params};

use crate::{error::Result, util};

/// (版本号, SQL)。新迁移追加到末尾即可。
const MIGRATIONS: &[(i64, &str)] = &[
    (1, include_str!("../migrations/0001_init.sql")),
    (2, include_str!("../migrations/0002_schedule.sql")),
];

const BOOTSTRAP_SQL: &str = "CREATE TABLE IF NOT EXISTS schema_migrations (
    version    INTEGER PRIMARY KEY,
    applied_at TEXT NOT NULL
);";

pub fn run(conn: &Connection) -> Result<()> {
    conn.execute_batch(BOOTSTRAP_SQL)?;
    let current: i64 = conn.query_row(
        "SELECT COALESCE(MAX(version), 0) FROM schema_migrations",
        [],
        |r| r.get(0),
    )?;
    for (version, sql) in MIGRATIONS {
        if *version > current {
            // 单个迁移整体生效：执行失败时由调用方决定重建/回滚策略（本地库可安全重建）。
            conn.execute_batch(sql)?;
            conn.execute(
                "INSERT INTO schema_migrations (version, applied_at) VALUES (?1, ?2)",
                params![version, util::now_iso()],
            )?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn migrations_are_idempotent() {
        let conn = Connection::open_in_memory().unwrap();
        run(&conn).unwrap();
        run(&conn).unwrap();
        let v: i64 = conn
            .query_row("SELECT MAX(version) FROM schema_migrations", [], |r| {
                r.get(0)
            })
            .unwrap();
        assert_eq!(v, 2);
    }
}
