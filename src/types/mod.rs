//! 类型定义模块
//!
//! 定义通用的数据类型和错误结构

use serde::{Deserialize, Serialize};

/// 标准错误返回结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse {
    /// 错误码
    pub errno: i32,
    /// 错误消息
    pub msg: String,
    /// 可选的详细信息
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<serde_json::Value>,
}

impl ErrorResponse {
    /// 创建新的错误响应
    pub fn new(errno: i32, msg: String, detail: Option<serde_json::Value>) -> Self {
        Self {
            errno,
            msg,
            detail,
        }
    }

    /// 转换为JSON值
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::to_value(self).unwrap_or_default()
    }
}

/// 常用错误码定义
pub mod error_codes {
    /// 请求错误
    pub const BAD_REQUEST: i32 = 400;
    /// 配置错误
    pub const CONFIG_ERROR: i32 = 400;
    /// 条件不满足
    pub const CONDITION_NOT_MET: i32 = 401;
    /// 数据验证失败
    pub const VALIDATION_FAILED: i32 = 402;
    /// 禁止访问
    pub const FORBIDDEN: i32 = 403;
    /// 处理超时
    pub const TIMEOUT: i32 = 408;
    /// 内部错误
    pub const INTERNAL_ERROR: i32 = 500;
    /// 未找到挂件
    pub const CHAINWARE_NOT_FOUND: i32 = 404;
}
