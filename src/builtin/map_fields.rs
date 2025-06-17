//! 字段映射挂件
//!
//! 将对象的字段进行重命名和转换

use crate::chainware::core::Chainware;
use crate::chainware::config::ChainwareConfig;
use crate::core::{ExecutionStatus, RequestContext, ResponseContext};
use crate::types::{error_codes, ErrorResponse};
use crate::utils::json_path::JsonPathTemplate;
use serde_json::Value;

/// 字段映射挂件
pub struct MapFieldsChainware {
    name: String,
}

impl Default for MapFieldsChainware {
    fn default() -> Self {
        Self::new()
    }
}

impl MapFieldsChainware {
    pub fn new() -> Self {
        Self {
            name: "map_fields".to_string(),
        }
    }

    /// 字段映射：将对象的字段进行重命名和转换
    /// 
    /// 业务逻辑：
    /// - overwrite=true: 在原对象基础上添加映射字段，容错性强
    /// - overwrite=false: 创建新对象只包含映射字段，严格模式
    fn process_map_fields(&self, input: &Value, mappings: &Value, context: &Value, overwrite: bool) -> Result<Value, String> {
        // 第一步：验证映射配置
        let mapping_obj = match mappings.as_object() {
            Some(obj) => obj,
            None => {
                if overwrite {
                    // overwrite模式：配置错误时容错，返回原数据
                    return Ok(input.clone());
                } else {
                    // 严格模式：配置错误时返回错误
                    return Err("映射配置必须是对象类型".to_string());
                }
            }
        };

        if overwrite {
            // 业务场景：增强原对象，保留所有原始数据
            self.process_overwrite_mode(input, mapping_obj, context)
        } else {
            // 业务场景：数据转换，只输出映射后的数据
            self.process_strict_mode(input, mapping_obj, context)
        }
    }

    /// overwrite=true模式：在原对象/数组基础上添加映射字段
    /// 适用场景：数据增强、字段别名、保持原数据完整性
    fn process_overwrite_mode(&self, input: &Value, mapping_obj: &serde_json::Map<String, Value>, context: &Value) -> Result<Value, String> {
        match input {
            // 处理对象：复制原对象，添加映射字段
            Value::Object(input_obj) => {
                let mut result = input_obj.clone();

                // 应用字段映射
                for (new_field, source_path) in mapping_obj {
                    if let Some(path_str) = source_path.as_str() {
                        match JsonPathTemplate::get_value(context, path_str) {
                            Ok(value) => {
                                // 添加或覆盖字段
                                result.insert(new_field.to_string(), value);
                            }
                            Err(err) => {
                                // 映射失败时返回错误
                                return Err(format!("字段映射失败，路径 '{}': {}", path_str, err));
                            }
                        }
                    } else {
                        // 映射值不是字符串时跳过，保持容错性
                        continue;
                    }
                }

                Ok(Value::Object(result))
            }
            // 处理数组：循环处理数组中的每个元素
            Value::Array(input_array) => {
                let mut result_array = Vec::new();

                for item in input_array {
                    if let Value::Object(item_obj) = item {
                        // 数组元素是对象，进行字段映射
                        let mut result_item = item_obj.clone();

                        // 应用字段映射
                        for (new_field, source_path) in mapping_obj {
                            if let Some(path_str) = source_path.as_str() {
                                // 为数组元素构建独立的context，将当前元素作为$.input
                                let item_context = serde_json::json!({
                                    "__input": item,
                                    "__params": context["__params"],
                                    "__meta": context["__meta"]
                                });
                                
                                match JsonPathTemplate::get_value(&item_context, path_str) {
                                    Ok(value) => {
                                        result_item.insert(new_field.to_string(), value);
                                    }
                                    Err(err) => {
                                        return Err(format!("数组元素字段映射失败，路径 '{}': {}", path_str, err));
                                    }
                                }
                            }
                        }

                        result_array.push(Value::Object(result_item));
                    } else {
                        // 数组元素不是对象，直接保留原值
                        result_array.push(item.clone());
                    }
                }

                Ok(Value::Array(result_array))
            }
            // 其他类型：直接返回原数据
            _ => Ok(input.clone()),
        }
    }

    /// overwrite=false模式：创建新对象，只包含映射的字段
    /// 适用场景：数据提取、格式转换、清理无关数据
    fn process_strict_mode(&self, _input: &Value, mapping_obj: &serde_json::Map<String, Value>, context: &Value) -> Result<Value, String> {
        // 建立新对象，根据map设置字段，返回新对象
        let mut result = serde_json::Map::new();

        // 只处理映射的字段
        for (new_field, source_path) in mapping_obj {
            if let Some(path_str) = source_path.as_str() {
                match JsonPathTemplate::get_value(context, path_str) {
                    Ok(value) => {
                        // 只添加成功映射的字段
                        result.insert(new_field.to_string(), value);
                    }
                    Err(err) => {
                        // 严格模式下任何映射失败都返回错误
                        return Err(format!("字段映射失败，路径 '{}': {}", path_str, err));
                    }
                }
            } else {
                // 严格模式下配置错误返回错误
                return Err(format!("映射配置错误，字段 '{}' 的值必须是字符串", new_field));
            }
        }

        Ok(Value::Object(result))
    }
}

impl Chainware for MapFieldsChainware {
    fn name(&self) -> &str {
        &self.name
    }

    fn process(
        &self,
        request: &RequestContext,
        response: &mut ResponseContext,
        data: Option<serde_json::Value>,
        config: Option<&ChainwareConfig>,
    ) -> Option<serde_json::Value> {
        let input = data.unwrap_or_default();

        // 获取映射配置
        let (mappings, overwrite) = match config {
            Some(cfg) => {
                let mappings = match cfg.config.get("mapping") {
                    Some(mappings) => mappings,
                    None => {
                        // 如果没有mapping配置，返回原数据
                        return Some(input);
                    }
                };
                let overwrite = cfg.config.get("overwrite")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(true); // 默认为true（保持原有行为）
                (mappings, overwrite)
            }
            None => {
                // 如果没有配置，返回原数据
                return Some(input);
            }
        };

        // 构建完整的上下文对象
        let context = JsonPathTemplate::build_context(&input, request);

        match self.process_map_fields(&input, mappings, &context, overwrite) {
            Ok(result) => Some(result),
            Err(err) => {
                response.status = ExecutionStatus::Error;
                response.data = Some(
                    ErrorResponse::new(
                        error_codes::INTERNAL_ERROR,
                        format!("字段映射失败: {}", err),
                        None,
                    )
                    .to_json(),
                );
                None
            }
        }
    }
}
