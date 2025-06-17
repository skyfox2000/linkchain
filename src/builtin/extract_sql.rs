use regex::Regex;
use serde_json::Value;
use std::sync::OnceLock;

use crate::chainware::core::Chainware;
use crate::chainware::config::ChainwareConfig;
use crate::core::{RequestContext, ResponseContext};
use crate::types::{error_codes, ErrorResponse};

/// SQL语句提取挂件
/// 从文本内容中提取SQL语句，支持markdown ```sql语法包裹和直接的SQL语句
pub struct ExtractSqlChainware {
    pub name: String,
}

impl Default for ExtractSqlChainware {
    fn default() -> Self {
        Self::new()
    }
}

// 全局正则表达式模式
static SQL_PATTERN: OnceLock<Regex> = OnceLock::new();
static SQL_STATEMENT: OnceLock<Regex> = OnceLock::new();

impl ExtractSqlChainware {
    /// 创建新的SQL提取挂件
    pub fn new() -> Self {
        Self { 
            name: "extract_sql".to_string() 
        }
    }

    /// 从文本中提取第一个SQL语句
    fn extract_sql_from_text(&self, text: &str) -> Result<Value, String> {
        // 初始化正则表达式模式
        let sql_pattern = SQL_PATTERN.get_or_init(|| {
            Regex::new(r"```sql\s*([\s\S]*?)\s*```").unwrap()
        });

        let sql_statement = SQL_STATEMENT.get_or_init(|| {
            Regex::new(r"(?i)(SELECT|INSERT|UPDATE|DELETE|CREATE|ALTER|DROP|TRUNCATE|GRANT|REVOKE|COMMIT|ROLLBACK)\s+[\s\S]+?;").unwrap()
        });

        // 首先尝试匹配Markdown SQL代码块
        for cap in sql_pattern.captures_iter(text) {
            if let Some(sql) = cap.get(1) {
                let sql_text = sql.as_str().trim();
                if !sql_text.is_empty() {
                    return Ok(Value::String(sql_text.to_string()));
                }
            }
        }

        // 如果没有找到代码块，尝试直接匹配SQL语句
        for cap in sql_statement.captures_iter(text) {
            if let Some(sql) = cap.get(0) {
                let sql_text = sql.as_str().trim();
                if !sql_text.is_empty() {
                    return Ok(Value::String(sql_text.to_string()));
                }
            }
        }

        // 没有找到SQL语句，返回Null
        Ok(Value::Null)
    }
}

impl Chainware for ExtractSqlChainware {
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
        // 获取输入数据
        let input = data.unwrap_or(Value::Null);

        // 获取文本内容 - 只支持字符串类型
        let text = match &input {
            Value::String(s) => s.as_str(),
            _ => {
                // 非字符串类型返回 Null
                response.data = Some(Value::Null);
                return Some(Value::Null);
            }
        };

        // 提取SQL语句
        match self.extract_sql_from_text(text) {
            Ok(result) => {
                response.data = Some(result.clone());
                Some(result)
            }
            Err(err) => {
                response.data = Some(
                    ErrorResponse::new(
                        error_codes::INTERNAL_ERROR,
                        format!("SQL提取失败: {}", err),
                        None,
                    )
                    .to_json(),
                );
                None
            }
        }
    }
} 