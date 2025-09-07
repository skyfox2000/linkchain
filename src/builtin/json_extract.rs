//! 数据提取挂件
//!
//! 基于JSONPath从对象或数组中提取指定内容

use crate::chainware::core::Chainware;
use crate::chainware::config::ChainwareConfig;
use crate::core::{ChainRequest, ChainResponse};
use crate::types::{error_codes, ErrorResponse};
use crate::utils::json_path::JsonPathTemplate;
use serde_json::Value;

/// 数据提取挂件
pub struct JsonExtractChainware {
    name: String,
}

impl Default for JsonExtractChainware {
    fn default() -> Self {
        Self::new()
    }
}

impl JsonExtractChainware {
    pub fn new() -> Self {
        Self {
            name: "json_extract".to_string(),
        }
    }

    /// 提取操作：基于JSONPath从对象或数组中提取指定内容
    fn process_extract(&self, input: &Value, pattern: Option<&Value>, context: &Value) -> Result<Value, String> {
        // 如果pattern为空，返回原data
        let pattern_str = match pattern {
            Some(Value::String(path)) => path,
            _ => return Ok(input.clone()),
        };

        // 使用统一的JsonPathTemplate工具获取值
        JsonPathTemplate::get_value(context, pattern_str)
    }
}

impl Chainware for JsonExtractChainware {
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

        // 构建完整的上下文对象
        let context = JsonPathTemplate::build_context(&input, request);

        // 获取提取参数
        let pattern = config.and_then(|cfg| cfg.config.get("pattern"));

        match self.process_extract(&input, pattern, &context) {
            Ok(result) => Some(result),
            Err(err) => {
                response.data = Some(
                    ErrorResponse::new(
                        error_codes::INTERNAL_ERROR,
                        format!("提取操作失败: {}", err),
                        None,
                    )
                    .to_json(),
                );
                None
            }
        }
    }
}
