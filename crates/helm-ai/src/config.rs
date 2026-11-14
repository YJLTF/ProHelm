//! 模型接入配置。全部通过 OpenAI 兼容协议访问（/chat/completions）。

/// 供应商接入配置：base_url + key + 模型名。
#[derive(Debug, Clone)]
pub struct AiProviderConfig {
    pub base_url: String,
    pub api_key: String,
    pub model: String,
}

impl AiProviderConfig {
    pub fn new(
        base_url: impl Into<String>,
        api_key: impl Into<String>,
        model: impl Into<String>,
    ) -> Self {
        Self {
            base_url: base_url.into(),
            api_key: api_key.into(),
            model: model.into(),
        }
    }

    /// 归一化出 chat/completions 完整端点：
    /// - 已是完整端点 → 原样；
    /// - 以版本段结尾（/v1、/v4 等）→ 追加 /chat/completions；
    /// - 其余 → 追加 /v1/chat/completions。
    pub fn chat_url(&self) -> String {
        chat_url_for(&self.base_url)
    }
}

pub fn chat_url_for(base_url: &str) -> String {
    let base = base_url.trim_end_matches('/');
    if base.ends_with("/chat/completions") {
        return base.to_string();
    }
    let last_segment = base.rsplit('/').next().unwrap_or("");
    let ends_with_version = last_segment
        .strip_prefix('v')
        .is_some_and(|rest| !rest.is_empty() && rest.bytes().all(|b| b.is_ascii_digit()));
    if ends_with_version {
        format!("{base}/chat/completions")
    } else {
        format!("{base}/v1/chat/completions")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chat_url_normalization() {
        assert_eq!(
            chat_url_for("https://api.openai.com/v1"),
            "https://api.openai.com/v1/chat/completions"
        );
        assert_eq!(
            chat_url_for("https://open.bigmodel.cn/api/paas/v4"),
            "https://open.bigmodel.cn/api/paas/v4/chat/completions"
        );
        assert_eq!(
            chat_url_for("http://localhost:11434/v1/"),
            "http://localhost:11434/v1/chat/completions"
        );
        assert_eq!(
            chat_url_for("https://example.com/api"),
            "https://example.com/api/v1/chat/completions"
        );
        assert_eq!(
            chat_url_for("https://example.com/v1/chat/completions"),
            "https://example.com/v1/chat/completions"
        );
    }
}
