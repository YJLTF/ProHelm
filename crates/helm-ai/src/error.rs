use thiserror::Error;

/// AI 引擎统一错误。
#[derive(Debug, Error)]
pub enum AiError {
    #[error("网络请求失败: {0}")]
    Http(#[from] reqwest::Error),

    #[error("模型输出不符合预期: {0}")]
    BadOutput(String),

    #[error("密钥读写失败: {0}")]
    Keyring(String),

    #[error("AI 尚未配置: {0}")]
    NotConfigured(String),
}

pub type AiResult<T> = Result<T, AiError>;
