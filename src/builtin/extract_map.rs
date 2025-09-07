//! 提取映射挂件
//!
//! 从输入数据中提取指定字段，组成新对象返回

use crate::chainware::core::Chainware;
use crate::chainware::config::ChainwareConfig;
use crate::core::{ChainStatus, ChainRequest, ChainResponse};
use crate::types::{error_codes, ErrorResponse};
use crate::utils::json_path::JsonPathTemplate;
use serde_json::Value;

/// 提取映射挂件
/// 
/// 功能：根据配置的key/template映射，从input数据中提取对应的值，组成新对象返回
/// 配置格式：
/// {
///   "mapping": {
///     "new_key1": "$.path.to.value1",    // 使用JSONPath提取值
///     "new_key2": "${template}",         // 使用模板字符串
///     "new_key3": "literal_value"        // 字面量值
///   }
/// }
pub struct ExtractMapChainware {
    name: String,
}

impl Default for ExtractMapChainware {
    fn default() -> Self {
        Self::new()
    }
}

impl ExtractMapChainware {
    pub fn new() -> Self {
        Self {
            name: "extract_map".to_string(),
        }
    }

    /// 执行提取映射：根据配置从输入数据中提取值组成新对象
    /// 
    /// # 参数
    /// - `input`: 输入数据
    /// - `mappings`: 映射配置对象
    /// - `context`: 完整的上下文对象（包含$input, $params, $meta等）
    /// 
    /// # 返回
    /// - 成功：包含提取值的新对象
    /// - 失败：错误信息
    fn process_extract_map(&self, _input: &Value, mappings: &Value, context: &Value) -> Result<Value, String> {
        // 验证映射配置必须是对象
        let mapping_obj = match mappings.as_object() {
            Some(obj) => obj,
            None => {
                return Err("mapping配置必须是对象类型".to_string());
            }
        };

        // 如果映射配置为空，返回空对象
        if mapping_obj.is_empty() {
            return Ok(Value::Object(serde_json::Map::new()));
        }

        let mut result = serde_json::Map::new();

        // 处理每个映射配置
        for (new_key, template_value) in mapping_obj {
            let extracted_value = match template_value {
                // 字符串类型：可能是JSONPath、模板或字面量
                Value::String(template) => {
                    self.extract_value_from_template(template, context)?
                }
                // 其他类型：直接作为字面量值使用
                _ => template_value.clone(),
            };

            result.insert(new_key.clone(), extracted_value);
        }

        Ok(Value::Object(result))
    }

    /// 从模板字符串中提取值
    /// 
    /// # 支持的模板格式
    /// - JSONPath: `$.path.to.value` 或 `$input.field` 或 `$params.key`
    /// - 模板字符串: `"Hello ${.name}!"` 
    /// - 字面量: `"literal_string"`
    fn extract_value_from_template(&self, template: &str, context: &Value) -> Result<Value, String> {
        let template = template.trim();

        // 情况1：以$开头的JSONPath路径 或 包含${}变量的模板字符串
        if template.starts_with('$') || template.contains("${") {
            match JsonPathTemplate::get_value(context, template) {
                Ok(Some(value)) => Ok(value),
                Ok(None) => Ok(Value::String(template.to_string())),
                Err(e) => Err(e),
            }
        }

        // 情况2：字面量字符串（不包含任何变量）
        else {
            Ok(Value::String(template.to_string()))
        }
    }

    /// 验证配置是否有效
    fn validate_config(&self, config: &ChainwareConfig) -> Result<Value, String> {
        match config.config.get("mapping") {
            Some(mappings) => {
                // 验证mapping是对象类型
                if !mappings.is_object() {
                    return Err("mapping配置必须是对象类型".to_string());
                }
                Ok(mappings.clone())
            }
            None => {
                Err("缺少必需的mapping配置".to_string())
            }
        }
    }
}

impl Chainware for ExtractMapChainware {
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

        // 验证配置
        let mappings = match config {
            Some(cfg) => {
                match self.validate_config(cfg) {
                    Ok(mappings) => mappings,
                    Err(err) => {
                        response.status = ChainStatus::Error;
                        response.data = Some(
                            ErrorResponse::new(
                                error_codes::CONFIG_ERROR,
                                format!("extract_map配置错误: {}", err),
                                Some(input),
                            )
                            .to_json(),
                        );
                        return None;
                    }
                }
            }
            None => {
                response.status = ChainStatus::Error;
                response.data = Some(
                    ErrorResponse::new(
                        error_codes::CONFIG_ERROR,
                        "extract_map挂件需要mapping配置".to_string(),
                        Some(input),
                    )
                    .to_json(),
                );
                return None;
            }
        };

        // 构建完整的上下文对象
        let context = JsonPathTemplate::build_context(&input, request);

        // 执行提取映射
        match self.process_extract_map(&input, &mappings, &context) {
            Ok(result) => Some(result),
            Err(err) => {
                response.status = ChainStatus::Error;
                response.data = Some(
                    ErrorResponse::new(
                        error_codes::INTERNAL_ERROR,
                        format!("extract_map处理失败: {}", err),
                        Some(input),
                    )
                    .to_json(),
                );
                None
            }
        }
    }
} 