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

    /// 获取字符串参数
    pub fn get_string_param(&self, key: &str) -> Option<String> {
        self.config.get(key)?.as_str().map(|s| s.to_string())
    }

    /// 获取数字参数
    pub fn get_number_param(&self, key: &str) -> Option<f64> {
        self.config.get(key)?.as_f64()
    }

    /// 获取布尔参数
    pub fn get_bool_param(&self, key: &str) -> Option<bool> {
        self.config.get(key)?.as_bool()
    }

    /// 获取数组参数
    pub fn get_array_param(&self, key: &str) -> Option<&Vec<serde_json::Value>> {
        self.config.get(key)?.as_array()
    }

    /// 获取对象参数
    pub fn get_object_param(
        &self,
        key: &str,
    ) -> Option<&serde_json::Map<String, serde_json::Value>> {
        self.config.get(key)?.as_object()
    }
} 