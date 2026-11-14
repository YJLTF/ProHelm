//! 键值设置仓库（非敏感配置；API Key 走 OS 钥匙串，绝不落库）。

use rusqlite::{Connection, params};

use crate::error::Result;

pub fn get(conn: &Connection, key: &str) -> Result<Option<String>> {
    match conn.query_row(
        "SELECT value FROM settings WHERE key = ?1",
        params![key],
        |r| r.get::<_, String>(0),
    ) {
        Ok(v) => Ok(Some(v)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e.into()),
    }
}

pub fn set(conn: &Connection, key: &str, value: &str) -> Result<()> {
    conn.execute(
        "INSERT INTO settings (key, value) VALUES (?1, ?2)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        params![key, value],
    )?;
    Ok(())
}
