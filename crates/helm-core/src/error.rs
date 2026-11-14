use thiserror::Error;

/// 核心层统一错误类型。
#[derive(Debug, Error)]
pub enum HelmError {
    #[error("数据库错误: {0}")]
    Db(#[from] rusqlite::Error),

    #[error("参数校验失败: {0}")]
    Validation(String),

    #[error("资源不存在: {0}")]
    NotFound(String),

    #[error("序列化错误: {0}")]
    Json(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, HelmError>;
