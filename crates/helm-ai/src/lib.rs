//! helm-ai：ProHelm 的 AI 引擎。
//!
//! 设计（见 docs/03-技术选型与架构设计.md 第 6 章）：
//! - 统一走 OpenAI 兼容协议（覆盖 GLM/DeepSeek/Qwen/OpenRouter/Ollama 等）；
//! - API Key 存 OS 钥匙串，绝不落库明文；
//! - PMP 知识库自研撰写（规避 PMBOK 原文版权），v1 用关键词评分检索，
//!   S6 升级为向量检索；
//! - 一切生成结果以 JSON/文本返回给命令层，由命令层解析为领域对象并
//!   记录 AiArtifact 溯源。

pub mod config;
pub mod error;
pub mod gateway;
pub mod kb;
pub mod keys;

pub use config::AiProviderConfig;
pub use error::{AiError, AiResult};
pub use gateway::Message;
