//! 上下文模块
//!
//! 定义请求上下文和响应上下文

use crate::core::status::ExecutionStatus;
use crate::core::utils;
use std::collections::HashMap;

/// 请求上下文
/// 包含执行链所需的所有输入信息
#[derive(Debug, Clone)]
pub struct RequestContext {
    /// 执行ID，用于追踪
    pub execution_id: String,
    /// 输入数据（JSON格式）
    pub params: serde_json::Value,
    /// 元数据（包含所有额外信息）
    pub meta: HashMap<String, serde_json::Value>,
    /// 执行开始时间
    pub start_time: u64,
}

impl RequestContext {
    /// 创建新的请求上下文
    pub fn new(params: serde_json::Value) -> Self {
        Self {
            execution_id: utils::generate_execution_id(),
            params,
            meta: HashMap::new(),
            start_time: utils::current_timestamp_ms(),
        }
    }

    /// 获取元数据
    pub fn get_meta(&self, key: &str) -> Option<&serde_json::Value> {
        self.meta.get(key)
    }
}

/// 响应上下文
/// 包含执行结果和状态信息
#[derive(Debug, Clone)]
pub struct ResponseContext {
    /// 执行状态
    pub status: ExecutionStatus,
    /// 输出数据（JSON格式）
    pub data: Option<serde_json::Value>,
    /// 响应元数据
    pub meta: HashMap<String, serde_json::Value>,
    /// 执行结束时间
    pub end_time: u64,
}

impl ResponseContext {
    /// 创建新的响应上下文
    pub fn new() -> Self {
        Self {
            status: ExecutionStatus::Continue,
            data: None,
            meta: HashMap::new(),
            end_time: utils::current_timestamp_ms(),
        }
    }

    /// 设置执行状态
    pub fn set_status(&mut self, status: ExecutionStatus) {
        self.status = status;
    }

    /// 设置输出数据
    pub fn set_data(&mut self, data: serde_json::Value) {
        self.data = Some(data);
    }

    /// 设置元数据
    pub fn set_meta(&mut self, key: String, value: serde_json::Value) {
        self.meta.insert(key, value);
    }

    /// 获取元数据
    pub fn get_metadata(&self, key: &str) -> Option<&serde_json::Value> {
        self.meta.get(key)
    }

    /// 设置执行结束时间
    pub fn set_end_time(&mut self) {
        self.end_time = utils::current_timestamp_ms();
    }

    /// 获取执行结束时间
    pub fn get_end_time(&self) -> u64 {
        self.end_time
    }
}

impl Default for ResponseContext {
    fn default() -> Self {
        Self::new()
    }
}
