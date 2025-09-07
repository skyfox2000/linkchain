//! 正则条件挂件
//!
//! 用于测试数据是否符合某种正则规则，符合则通过，不符合则拒绝

use crate::chainware::core::Chainware;
use crate::chainware::config::ChainwareConfig;
use crate::core::{ChainStatus, ChainRequest, ChainResponse};
use crate::types::{error_codes, ErrorResponse};
use regex::Regex;
use serde_json::Value;

/// 正则条件挂件
pub struct RegexpConditionChainware {
    name: String,
}

impl Default for RegexpConditionChainware {
    fn default() -> Self {
        Self::new()
    }
}

impl RegexpConditionChainware {
    pub fn new() -> Self {
        Self {
            name: "regexp_condition".to_string(),
        }
    }

    /// 正则条件检查：测试数据是否符合正则规则
    fn process_regexp_condition(
        &self,
        input: &Value,
        pattern: Option<&str>,
    ) -> Result<bool, String> {
        // 如果pattern为空，返回true（通过检查）
        let pattern = match pattern {
            Some(p) => p,
            None => return Ok(true),
        };

        // 获取要检查的文本内容
        let text = match input {
            Value::String(s) => s.as_str(),
            _ => {
                // 将值转换为字符串进行匹配
                &input.to_string()
            }
        };

        // 自动为正则表达式添加^$，确保完全匹配
        let full_pattern = if pattern.starts_with('^') && pattern.ends_with('$') {
            pattern.to_string()
        } else if pattern.starts_with('^') {
            format!("{}$", pattern)
        } else if pattern.ends_with('$') {
            format!("^{}", pattern)
        } else {
            format!("^{}$", pattern)
        };

        // 编译正则表达式
        let regex = match Regex::new(&full_pattern) {
            Ok(r) => r,
            Err(e) => return Err(format!("正则表达式编译失败: {}", e)),
        };

        // 执行匹配
        Ok(regex.is_match(text))
    }
}

impl Chainware for RegexpConditionChainware {
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

        match self.process_regexp_condition(&input, pattern) {
            Ok(true) => {
                // 正则匹配成功，返回传入的数据
                Some(input)
            }
            Ok(false) => {
                // 正则匹配失败，设置拒绝状态
                response.status = ChainStatus::Reject;
                response.data = Some(
                    ErrorResponse::new(
                        error_codes::CONDITION_NOT_MET,
                        format!("数据不符合正则规则: {}", pattern.unwrap_or("无")),
                        None,
                    )
                    .to_json(),
                );
                None
            }
            Err(err) => {
                // 正则处理错误
                response.status = ChainStatus::Error;
                response.data = Some(
                    ErrorResponse::new(
                        error_codes::INTERNAL_ERROR,
                        format!("正则条件检查失败: {}, {}", pattern.unwrap_or("N/A"), err),
                        None,
                    )
                    .to_json(),
                );
                None
            }
        }
    }
}
