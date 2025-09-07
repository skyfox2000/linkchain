//! 节点配置模块
//!
//! 定义节点配置结构

use std::collections::HashMap;

/// 节点配置
/// 通用配置结构，支持任意参数
#[derive(Debug, Clone)]
pub struct ChainwareConfig {
    /// 配置参数（通用参数，支持任意类型）
    pub config: HashMap<String, serde_json::Value>,
}

impl ChainwareConfig {
    /// 创建新的节点配置
    pub fn new(config: HashMap<String, serde_json::Value>) -> Self {
        Self { config }
    }

    /// 获取参数
    pub fn get_param(&self, key: &str) -> Option<&serde_json::Value> {
        self.config.get(key)
    }

    /// 获取启用状态
    pub fn get_enabled(&self) -> bool {
        self.config.get("enabled").unwrap_or(&serde_json::Value::Bool(true)).as_bool().unwrap_or(true)
    }
} 