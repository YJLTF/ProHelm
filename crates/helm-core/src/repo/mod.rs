//! 仓库层：SQL 与领域对象的唯一边界。

pub mod baselines;
pub mod dependencies;
pub mod oplog;
pub mod projects;
pub mod schedule;
pub mod stats;
pub mod tasks;

use rusqlite::Error as SqlError;
use rusqlite::types::Type;

use crate::error::HelmError;

/// 把领域枚举解析失败转换为 rusqlite 行映射错误。
pub(crate) fn enum_conversion_error(e: HelmError) -> SqlError {
    SqlError::FromSqlConversionFailure(0, Type::Text, Box::new(e))
}
