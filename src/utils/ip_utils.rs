//! IP工具函数
//!
//! 提供IP相关的通用处理函数

use serde_json::Value;

/// 从配置值中提取IP列表
/// 
/// 支持两种格式：
/// - 字符串：逗号分隔的IP列表
/// - 数组：字符串数组形式的IP列表
/// 
/// 返回：Vec<String> 包含所有有效的IP地址
pub fn extract_ip_list(config_value: Option<&Value>) -> Result<Vec<String>, String> {
    match config_value {
        Some(Value::String(ip_str)) => {
            // 支持逗号分割的字符串格式
            Ok(ip_str
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect::<Vec<String>>())
        }
        Some(Value::Array(ip_array)) => {
            // 支持数组格式
            let mut result = Vec::new();
            for item in ip_array {
                match item {
                    Value::String(ip) => {
                        let trimmed_ip = ip.trim();
                        if !trimmed_ip.is_empty() {
                            result.push(trimmed_ip.to_string());
                        }
                    }
                    _ => {
                        return Err("IP列表数组中的元素必须是字符串类型".to_string());
                    }
                }
            }
            Ok(result)
        }
        Some(_) => {
            Err("ip_list配置必须是字符串或数组类型".to_string())
        }
        None => {
            Err("缺少ip_list配置".to_string())
        }
    }
}