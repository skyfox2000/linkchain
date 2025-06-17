//! JSON提取挂件
//!
//! 从文本中提取JSON对象

use crate::chainware::core::Chainware;
use crate::chainware::config::ChainwareConfig;
use crate::core::{RequestContext, ResponseContext};
use crate::types::{error_codes, ErrorResponse};
use serde_json::Value;

/// JSON提取挂件
pub struct ExtractJsonChainware {
    name: String,
}

impl Default for ExtractJsonChainware {
    fn default() -> Self {
        Self::new()
    }
}

impl ExtractJsonChainware {
    pub fn new() -> Self {
        Self {
            name: "extract_json".to_string(),
        }
    }

    /// 提取JSON：从文本中提取JSON对象或数组
    fn process_extract_json(&self, input: &Value) -> Result<Value, String> {
        // 获取文本内容
        let text = match input {
            Value::String(s) => s.as_str(),
            Value::Object(obj) => &serde_json::to_string(obj).map_err(|e| format!("对象转字符串失败: {}", e))?,
            _ => return Err("提取JSON需要字符串输入".to_string()),
        };

        // 按照文本中出现的顺序处理 JSON 对象和数组
        for (i, c) in text.char_indices() {
            if c == '{' {
                // 尝试解析JSON对象
                if let Ok(json_value) = self.try_parse_json_object(text, i) {
                    return Ok(json_value);
                }
            } else if c == '[' {
                // 尝试解析JSON数组
                if let Ok(json_value) = self.try_parse_json_array(text, i) {
                    return Ok(json_value);
                }
            }
        }

        // 没有找到有效的JSON
        Ok(Value::Null)
    }

    /// 尝试从指定位置解析JSON对象
    fn try_parse_json_object(&self, text: &str, start_pos: usize) -> Result<Value, String> {
        let mut brace_count = 0;
        let mut end_pos = None;

        // 从开始位置查找匹配的结束位置
        for (i, c) in text[start_pos..].char_indices() {
            let actual_pos = start_pos + i;
            if c == '{' {
                brace_count += 1;
            } else if c == '}' {
                brace_count -= 1;
                if brace_count == 0 {
                    end_pos = Some(actual_pos);
                    break;
                }
            }
        }

        // 如果找到完整的对象，尝试解析
        if let Some(end) = end_pos {
            let json_str = &text[start_pos..=end];
            match serde_json::from_str::<Value>(json_str) {
                Ok(json_value) => Ok(json_value),
                Err(_) => Err("JSON对象解析失败".to_string()),
            }
        } else {
            Err("未找到完整的JSON对象".to_string())
        }
    }

    /// 尝试从指定位置解析JSON数组
    fn try_parse_json_array(&self, text: &str, start_pos: usize) -> Result<Value, String> {
        let mut bracket_count = 0;
        let mut end_pos = None;

        // 从开始位置查找匹配的结束位置
        for (i, c) in text[start_pos..].char_indices() {
            let actual_pos = start_pos + i;
            if c == '[' {
                bracket_count += 1;
            } else if c == ']' {
                bracket_count -= 1;
                if bracket_count == 0 {
                    end_pos = Some(actual_pos);
                    break;
                }
            }
        }

        // 如果找到完整的数组，尝试解析
        if let Some(end) = end_pos {
            let json_str = &text[start_pos..=end];
            match serde_json::from_str::<Value>(json_str) {
                Ok(json_value) => Ok(json_value),
                Err(_) => Err("JSON数组解析失败".to_string()),
            }
        } else {
            Err("未找到完整的JSON数组".to_string())
        }
    }
}

impl Chainware for ExtractJsonChainware {
    fn name(&self) -> &str {
        &self.name
    }

    fn process(
        &self,
        _request: &RequestContext,
        response: &mut ResponseContext,
        data: Option<serde_json::Value>,
        _config: Option<&ChainwareConfig>,
    ) -> Option<serde_json::Value> {
        let input = data.unwrap_or_default();

        match self.process_extract_json(&input) {
            Ok(result) => Some(result),
            Err(err) => {
                response.data = Some(
                    ErrorResponse::new(
                        error_codes::INTERNAL_ERROR,
                        format!("JSON提取失败: {}", err),
                        None,
                    )
                    .to_json(),
                );
                None
            }
        }
    }
}
