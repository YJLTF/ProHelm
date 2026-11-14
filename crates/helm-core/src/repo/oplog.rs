//! 操作日志（OpLog）：所有写操作追加记录。
//!
//! M1 用途：本地操作审计；M3 用途：同步引擎的事件源；S4 用途：AI 变更撤销。

use rusqlite::{Connection, params};

use crate::{error::Result, util};

pub fn record(
    conn: &Connection,
    entity: &str,
    entity_id: &str,
    op: &str,
    payload: &str,
) -> Result<()> {
    conn.execute(
        "INSERT INTO oplog (id, ts, entity, entity_id, op, payload) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![util::new_id(), util::now_iso(), entity, entity_id, op, payload],
    )?;
    Ok(())
}
