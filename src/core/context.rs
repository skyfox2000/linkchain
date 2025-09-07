//! 上下文模块
//!
//! 定义请求上下文和响应上下文

use crate::core::status::ChainStatus;
use crate::core::utils;
use std::collections::HashMap;
use uuid::Uuid;

/// 请求上下文
/// 包含执行链所需的所有输入信息
#[derive(Debug, Clone)]
pub struct ChainRequest {
    /// 输入数据（JSON格式）
    pub params: serde_json::Value,
    /**
     * 元数据（包含所有额外信息）
     * 格式：
     * {
     *     "node_name": "节点名称",
     *     "node_id": "节点ID",
     *     "node_type": "节点类型",
     *     "tool_name": "工具名称",
     *     "trace_id": "全局跟踪id",
     * }
     */
    pub meta: HashMap<String, serde_json::Value>,
    /// 执行开始时间
    pub start_time: u64,
}

impl ChainRequest {
    /// 创建新的请求上下文
    pub fn new(params: serde_json::Value, meta: HashMap<String, serde_json::Value>) -> Self {
        let mut meta = meta;
        // 如果meta中没有span_id，则生成一个新的
        if !meta.contains_key("span_id") {
            let span_id = Uuid::new_v4().to_string();
            meta.insert("span_id".to_string(), serde_json::Value::String(span_id));
        }
        
        Self {
            params,
            meta,
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
pub struct ChainResponse {
    /// 执行状态
    pub status: ChainStatus,
    /// 输出数据（JSON格式）
    pub data: Option<serde_json::Value>,
    /// 响应元数据
    pub meta: HashMap<String, serde_json::Value>,
    /// 执行开始时间
    pub start_time: u64,
    /// 执行结束时间
    pub end_time: u64,
}

impl ChainResponse {
    /// 创建新的响应上下文
    pub fn new(start_time: u64) -> Self {
        Self {
            status: ChainStatus::Continue,
            data: None,
            meta: HashMap::new(),
            start_time,
            end_time: utils::current_timestamp_ms(),
        }
    }

    /// 设置执行状态
    pub fn set_status(&mut self, status: ChainStatus) {
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

    pub fn set_start_time(&mut self, start_time: u64) {
        self.start_time = start_time;
    }

    /// 设置执行结束时间
    pub fn set_end_time(&mut self) {
        self.end_time = utils::current_timestamp_ms();
    }
}