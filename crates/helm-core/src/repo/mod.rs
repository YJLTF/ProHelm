//! 仓库层：SQL 与领域对象的唯一边界。

pub mod ai_artifacts;
pub mod baselines;
pub mod charters;
pub mod dependencies;
pub mod evm;
pub mod oplog;
pub mod projects;
pub mod risks;
pub mod schedule;
pub mod settings;
pub mod stakeholders;
pub mod stats;
pub mod tasks;
pub mod time_log;

use rusqlite::Error as SqlError;
use rusqlite::types::Type;

use crate::error::HelmError;

/// 把领域枚举解析失败转换为 rusqlite 行映射错误。
pub(crate) fn enum_conversion_error(e: HelmError) -> SqlError {
    SqlError::FromSqlConversionFailure(0, Type::Text, Box::new(e))
}
