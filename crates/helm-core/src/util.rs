/// 当前时间的 ISO 8601 表示（UTC，毫秒精度），用于 `created_at` / `updated_at`。
pub fn now_iso() -> String {
    chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true)
}

/// 生成 ULID 主键：时间有序 + 离线无冲突，为 M3 同步预留。
pub fn new_id() -> String {
    ulid::Ulid::new().to_string()
}
