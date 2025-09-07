//! 数据合并挂件
//!
//! 将额外数据合并到结果中

use crate::chainware::config::ChainwareConfig;
use crate::chainware::core::Chainware;
use crate::core::{ChainStatus, ChainRequest, ChainResponse};
use crate::types::{error_codes, ErrorResponse};
use crate::utils::json_path::JsonPathTemplate;
use serde_json::Value;

/// 数据合并挂件
pub struct MergeChainware {
    name: String,
}

impl Default for MergeChainware {
    fn default() -> Self {
        Self::new()
    }
}

impl MergeChainware {
    pub fn new() -> Self {
        Self {
            name: "merge".to_string(),
        }
    }

    /// 合并操作：根据data_path获取数据进行合并
    ///
    /// 业务逻辑：
    /// - 对象 + 对象 = 合并对象
    /// - 数组 + 数组 = 合并数组
    /// - 其他情况 = 返回原数据
    fn process_merge(
        &self,
        input: &Value,
        data_path: &str,
        context: &Value,
    ) -> Result<Value, String> {
        // 使用JsonPathTemplate从context获取合并数据
        let merge_data = match JsonPathTemplate::get_value(context, data_path) {
            Ok(Some(data)) => data,
            Ok(None) => return Ok(input.clone()),
            Err(err) => {
                return Err(format!("无法从路径 '{}' 获取数据: {}", data_path, err));
            }
        };

        match (input, &merge_data) {
            // 对象合并对象
            (Value::Object(input_obj), Value::Object(merge_obj)) => {
                let mut result = input_obj.clone();

                // 将merge_obj的所有字段合并到result中
                for (key, value) in merge_obj {
                    result.insert(key.clone(), value.clone());
                }

                Ok(Value::Object(result))
            }
            // 数组合并数组
            (Value::Array(input_array), Value::Array(merge_array)) => {
                let mut result = input_array.clone();

                // 将merge_array的所有元素追加到result中
                for item in merge_array {
                    result.push(item.clone());
                }

                Ok(Value::Array(result))
            }
            // 其他情况：类型不匹配，返回原数据
            _ => Ok(input.clone()),
        }
    }
}

impl Chainware for MergeChainware {
    fn name(&self) -> &str {
        &self.name
    }

    fn process(
        &self,
        request: &ChainRequest,
        response: &mut ChainResponse,
        data: Option<serde_json::Value>,
        config: Option<&ChainwareConfig>,
    ) -> Option<serde_json::Value> {
        let input = data.unwrap_or_default();

        // 获取data_path配置
        let data_path = match config.and_then(|cfg| cfg.config.get("data_path")) {
            Some(Value::String(path)) => path,
            Some(_) => {
                response.status = ChainStatus::Error;
                response.data = Some(
                    ErrorResponse::new(
                        error_codes::CONFIG_ERROR,
                        "data_path配置必须是字符串类型".to_string(),
                        None,
                    )
                    .to_json(),
                );
                return Some(input);
            }
            None => {
                // 如果没有data_path配置，返回原数据
                return Some(input);
            }
        };

        // 检查是否存在自引用
        if data_path.starts_with("$input") || data_path.starts_with("$data") || (data_path == "$") {
            response.status = ChainStatus::Error;
            response.data = Some(
                ErrorResponse::new(
                    error_codes::CONFIG_ERROR,
                    format!("data_path不能自引用输入数据，禁止使用路径: {}", data_path),
                    None,
                )
                .to_json(),
            );
            return Some(input);
        }

        // 构建完整的上下文对象
        let context = JsonPathTemplate::build_context(&input, request);

        match self.process_merge(&input, data_path, &context) {
            Ok(result) => Some(result),
            Err(err) => {
                response.status = ChainStatus::Error;
                response.data = Some(
                    ErrorResponse::new(
                        error_codes::INTERNAL_ERROR,
                        format!("合并操作失败: {}", err),
                        None,
                    )
                    .to_json(),
                );
                Some(input) // 返回原数据
            }
        }
    }
}
