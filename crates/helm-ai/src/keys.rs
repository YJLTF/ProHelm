//! API Key 的 OS 钥匙串存取（Win 凭据管理器 / macOS Keychain / Linux Secret Service）。
//! Key 绝不写入数据库或配置文件。

use crate::error::{AiError, AiResult};

const SERVICE: &str = "prohelm";
const USER: &str = "ai-api-key";

fn entry() -> AiResult<keyring::Entry> {
    keyring::Entry::new(SERVICE, USER).map_err(|e| AiError::Keyring(e.to_string()))
}

/// 保存 Key；空字符串表示删除已存 Key。
pub fn store_api_key(key: &str) -> AiResult<()> {
    let entry = entry()?;
    if key.is_empty() {
        match entry.delete_credential() {
            Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
            Err(e) => Err(AiError::Keyring(e.to_string())),
        }
    } else {
        entry
            .set_password(key)
            .map_err(|e| AiError::Keyring(e.to_string()))
    }
}

/// 读取 Key；未配置返回 None。
pub fn load_api_key() -> AiResult<Option<String>> {
    let entry = entry()?;
    match entry.get_password() {
        Ok(k) => Ok(Some(k)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(e) => Err(AiError::Keyring(e.to_string())),
    }
}
