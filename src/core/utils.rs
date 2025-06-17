//! 工具函数模块
//! 
//! 提供基础工具函数

use std::time::{SystemTime, UNIX_EPOCH};

/// 默认超时时间（毫秒）
pub const DEFAULT_TIMEOUT_MS: u64 = 30000;

/// 生成执行ID
pub fn generate_execution_id() -> String {
    uuid::Uuid::new_v4().to_string()
}

/// 计算执行耗时（毫秒）
pub fn calculate_duration_ms(start_time: u64) -> u64 {
    current_timestamp_ms() - start_time
}

/// 获取当前时间戳（秒）
pub fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// 获取当前时间戳（毫秒）
pub fn current_timestamp_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
} 