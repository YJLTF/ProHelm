//! 统一模型网关：非流式 / 流式 / 结构化 JSON 三种调用形态。
//!
//! 兼容 OpenAI 协议的 SSE：`data: {"choices":[{"delta":{"content":"…"}}]}`。

use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::config::AiProviderConfig;
use crate::error::{AiError, AiResult};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: String,
}

impl Message {
    pub fn system(content: impl Into<String>) -> Self {
        Self {
            role: "system".into(),
            content: content.into(),
        }
    }
    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: "user".into(),
            content: content.into(),
        }
    }
    #[allow(dead_code)]
    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: "assistant".into(),
            content: content.into(),
        }
    }
}

fn body(cfg: &AiProviderConfig, messages: &[Message], stream: bool) -> Value {
    serde_json::json!({
        "model": cfg.model,
        "messages": messages,
        "stream": stream,
    })
}

async fn send(
    cfg: &AiProviderConfig,
    messages: &[Message],
    stream: bool,
) -> AiResult<reqwest::Response> {
    let client = reqwest::Client::new();
    let resp = client
        .post(cfg.chat_url())
        .bearer_auth(&cfg.api_key)
        .json(&body(cfg, messages, stream))
        .send()
        .await?
        .error_for_status()?;
    Ok(resp)
}

/// 非流式对话：返回完整回复文本。
pub async fn chat(cfg: &AiProviderConfig, messages: &[Message]) -> AiResult<String> {
    let resp = send(cfg, messages, false).await?;
    let v: Value = resp.json().await?;
    let content = v["choices"][0]["message"]["content"]
        .as_str()
        .unwrap_or_default()
        .to_string();
    if content.is_empty() {
        return Err(AiError::BadOutput("模型返回了空回复".into()));
    }
    Ok(content)
}

/// 流式对话：每个文本增量回调 `on_delta`，返回拼接后的完整回复。
pub async fn chat_stream(
    cfg: &AiProviderConfig,
    messages: &[Message],
    on_delta: &mut (dyn FnMut(&str) + Send),
) -> AiResult<String> {
    let resp = send(cfg, messages, true).await?;
    let mut full = String::new();
    let mut buffer = String::new();
    let mut stream = resp.bytes_stream();
    while let Some(chunk) = stream.next().await {
        let bytes = chunk?;
        buffer.push_str(&String::from_utf8_lossy(&bytes));
        while let Some(idx) = buffer.find('\n') {
            let line: String = buffer.drain(..=idx).collect();
            handle_sse_line(line.trim(), &mut full, on_delta);
        }
    }
    if full.is_empty() {
        return Err(AiError::BadOutput("流式响应中没有收到任何文本".into()));
    }
    Ok(full)
}

/// 解析一行 SSE 数据：提取增量文本并累计到 `full`。公开供测试。
pub fn handle_sse_line(line: &str, full: &mut String, on_delta: &mut (dyn FnMut(&str) + Send)) {
    let Some(data) = line.strip_prefix("data:") else {
        return;
    };
    let data = data.trim();
    if data.is_empty() || data == "[DONE]" {
        return;
    }
    let Ok(v) = serde_json::from_str::<Value>(data) else {
        return;
    };
    if let Some(delta) = v["choices"][0]["delta"]["content"].as_str()
        && !delta.is_empty()
    {
        full.push_str(delta);
        on_delta(delta);
    }
}

/// 从模型输出中提取第一个 JSON 对象/数组文本块（容忍代码围栏与前后杂文本）。
pub fn extract_json_block(raw: &str) -> Option<&str> {
    let trimmed = raw.trim();
    let body = if let Some(rest) = trimmed.strip_prefix("```") {
        let rest = rest.strip_prefix("json").unwrap_or(rest);
        let inner = rest.trim_start();
        inner.strip_suffix("```").unwrap_or(inner).trim()
    } else {
        trimmed
    };

    let obj_start = body.find('{');
    let arr_start = body.find('[');
    let (open_ch, start) = match (obj_start, arr_start) {
        (Some(a), Some(b)) => {
            if a < b {
                ('{', a)
            } else {
                ('[', b)
            }
        }
        (Some(a), None) => ('{', a),
        (None, Some(b)) => ('[', b),
        (None, None) => return None,
    };
    let close_ch = if open_ch == '{' { '}' } else { ']' };
    let end = body.rfind(close_ch)?;
    if end < start {
        return None;
    }
    Some(&body[start..=end])
}

/// 结构化 JSON 任务：非流式对话 + 提取 JSON 块 + 解析。
pub async fn chat_json(cfg: &AiProviderConfig, system: &str, user: &str) -> AiResult<Value> {
    let raw = chat(cfg, &[Message::system(system), Message::user(user)]).await?;
    let block = extract_json_block(&raw)
        .ok_or_else(|| AiError::BadOutput("输出中未找到 JSON 内容".into()))?;
    serde_json::from_str(block).map_err(|e| AiError::BadOutput(format!("JSON 解析失败: {e}")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    fn collect_deltas(lines: &[&str]) -> (String, Vec<String>) {
        let full = Mutex::new(String::new());
        let deltas = Mutex::new(Vec::new());
        {
            let mut on_delta = |d: &str| {
                deltas.lock().unwrap().push(d.to_string());
            };
            for line in lines {
                handle_sse_line(line, &mut full.lock().unwrap(), &mut on_delta);
            }
        }
        (full.into_inner().unwrap(), deltas.into_inner().unwrap())
    }

    #[test]
    fn sse_delta_extraction() {
        let lines = [
            ": ping",
            "data: {\"choices\":[{\"delta\":{\"content\":\"你好\"}}]}",
            "data: {\"choices\":[{\"delta\":{\"content\":\"，世界\"}}]}",
            "data: [DONE]",
            "",
        ];
        let (full, deltas) = collect_deltas(&lines);
        assert_eq!(full, "你好，世界");
        assert_eq!(deltas, vec!["你好".to_string(), "，世界".to_string()]);
    }

    #[test]
    fn sse_ignores_malformed_lines() {
        let lines = ["data: not-json", "data: {}", "data: {\"other\":1}"];
        let (full, deltas) = collect_deltas(&lines);
        assert!(full.is_empty());
        assert!(deltas.is_empty());
    }

    #[test]
    fn extract_json_plain_and_fenced() {
        assert_eq!(extract_json_block("{\"a\":1}"), Some("{\"a\":1}"));
        assert_eq!(
            extract_json_block("```json\n[{\"t\":1}]\n```"),
            Some("[{\"t\":1}]")
        );
        assert_eq!(
            extract_json_block("好的，如下：\n{\"objective\":\"x\"}\n以上。"),
            Some("{\"objective\":\"x\"}")
        );
        assert_eq!(extract_json_block("没有 JSON"), None);
    }
}
