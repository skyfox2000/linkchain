//! 正则提取挂件
//!
//! 通过正则表达式从文本中提取内容，仅支持单个匹配内容的返回值

use crate::chainware::config::ChainwareConfig;
use crate::chainware::core::Chainware;
use crate::core::{ChainStatus, ChainRequest, ChainResponse};
use crate::types::{error_codes, ErrorResponse};
use regex::Regex;
use serde_json::Value;

/// 正则提取挂件
pub struct RegexpExtractChainware {
    name: String,
}

impl Default for RegexpExtractChainware {
    fn default() -> Self {
        Self::new()
    }
}

impl RegexpExtractChainware {
    pub fn new() -> Self {
        Self {
            name: "regexp_extract".to_string(),
        }
    }

    /// 正则提取：通过正则表达式从文本中提取内容
    fn process_regexp_extract(
        &self,
        input: &Value,
        pattern: Option<&str>,
    ) -> Result<Value, String> {
        // 如果pattern为空，返回原data
        let pattern = match pattern {
            Some(p) => p,
            None => return Ok(input.clone()),
        };

        // 获取文本内容
        let text = match input {
            Value::String(s) => s.as_str(),
            _ => return Err("正则提取需要字符串输入".to_string()),
        };

        // 编译正则表达式
        let regex = match Regex::new(pattern) {
            Ok(r) => r,
            Err(e) => return Err(format!("正则表达式编译失败: {}", e)),
        };

        // 执行匹配
        if let Some(captures) = regex.captures(text) {
            // 如果有捕获组，返回所有捕获组结果
            if captures.len() > 1 {
                let mut results = Vec::new();
                for i in 1..captures.len() {
                    if let Some(matched) = captures.get(i) {
                        results.push(Value::String(matched.as_str().to_string()));
                    } else {
                        results.push(Value::Null);
                    }
                }
                return Ok(Value::Array(results));
            }
            // 否则返回整个匹配
            if let Some(matched) = captures.get(0) {
                return Ok(Value::String(matched.as_str().to_string()));
            }
        }

        // 未找到匹配
        Ok(Value::Null)
    }
}

impl Chainware for RegexpExtractChainware {
    fn name(&self) -> &str {
        &self.name
    }

    fn process(
        &self,
        _request: &ChainRequest,
        response: &mut ChainResponse,
        data: Option<serde_json::Value>,
        config: Option<&ChainwareConfig>,
    ) -> Option<serde_json::Value> {
        let input = data.unwrap_or_default();

        // 从配置中获取正则表达式
        let pattern = match config.and_then(|cfg| cfg.config.get("pattern")) {
            Some(Value::String(p)) => Some(p.as_str()),
            Some(_) => {
                response.status = ChainStatus::Error;
                response.data = Some(
                    ErrorResponse::new(
                        error_codes::CONFIG_ERROR,
                        "配置中的pattern必须是字符串类型".to_string(),
                        None,
                    )
                    .to_json(),
                );
                return None;
            }
            None => None,
        };

        match self.process_regexp_extract(&input, pattern) {
            Ok(result) => Some(result),
            Err(err) => {
                response.status = ChainStatus::Error;
                response.data = Some(
                    ErrorResponse::new(
                        error_codes::INTERNAL_ERROR,
                        format!("正则提取失败: {}, {}", pattern.unwrap_or("N/A"), err),
                        None,
                    )
                    .to_json(),
                );
                None
            }
        }
    }
}
