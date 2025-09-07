//! 日志记录挂件
//!
//! 记录执行信息和数据状态

use crate::chainware::config::ChainwareConfig;
use crate::chainware::core::Chainware;
use crate::core::{ChainRequest, ChainResponse};
use crate::types::{error_codes, ErrorResponse};
use crate::utils::json_path::JsonPathTemplate;
use serde_json::Value;

/// 日志记录挂件
pub struct LoggerChainware {
    name: String,
}

impl Default for LoggerChainware {
    fn default() -> Self {
        Self::new()
    }
}

impl LoggerChainware {
    pub fn new() -> Self {
        Self {
            name: "logger".to_string(),
        }
    }

    /// 记录日志信息
    fn log_info(&self, context: &Value, template: &str) -> Result<(), String> {
        let message = JsonPathTemplate::get_value(context, template)?;
        println!("[Logger] {}", message);
        Ok(())
    }
}

impl Chainware for LoggerChainware {
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
        let context = JsonPathTemplate::build_context(&input, request);

        // 获取日志模板
        let template = match config.and_then(|cfg| cfg.config.get("template")) {
            Some(Value::String(tmpl)) => tmpl,
            _ => "打印日志: ${ data }",
        };

        // 记录日志
        if let Err(err) = self.log_info(&context, template) {
            response.data = Some(
                ErrorResponse::new(
                    error_codes::INTERNAL_ERROR,
                    format!("日志记录失败: {}", err),
                    None,
                )
                .to_json(),
            );
        }

        // 透传数据
        Some(input)
    }
}
