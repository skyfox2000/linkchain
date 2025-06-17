//! JSON路径模板工具
//!
//! 提供统一的JSON路径解析和模板替换功能

use jsonpath_rust::JsonPath;
use regex::Regex;
use serde_json::Value;

/// JSON路径模板工具
pub struct JsonPathTemplate;

impl JsonPathTemplate {
    /// 构建统一的上下文对象
    ///
    /// # 参数
    /// - `data`: 输入数据
    /// - `request`: 请求上下文
    pub fn build_context(data: &Value, request: &crate::core::RequestContext) -> Value {
        serde_json::json!({
            "__input": data,
            "__params": request.params,
            "__meta": request.meta
        })
    }

    /// 获取数据值
    ///
    /// # 参数
    /// - `context`: 上下文对象，包含data, params, meta
    /// - `template`: 模板字符串
    ///   - 如果以`$.`开头，则直接当作JSONPath使用，如果以`$[`开头，则当作数组索引使用
    ///   - 如果是`${}`格式，则使用{}内字符串作为路径获取数据
    ///   - 如果是单独的`$.`路径或完整的`${}`，则直接返回获取的数据
    ///   - 如果以`$params`开头，则当作params获取数据
    ///   - 如果以`$meta`开头，则当作meta获取数据
    ///   - 如果以`$data`开头，则当作input获取数据
    ///   - 如果以`$input`开头，则当作input获取数据
    ///   - 如果模板还有其它字符内容，则转换成字符串替换对应位置
    pub fn get_value(context: &Value, template: &str) -> Result<Value, String> {
        let template = template.trim();

        let (path, data) = {
            let processed_template = if let Some(stripped) = template.strip_prefix(".") {
                format!("$.{}", stripped)
            } else {
                template.to_string()
            };

            // 情况1：完整的${}变量
            let processed_template = if processed_template.starts_with("${")
                && processed_template.ends_with("}")
                && processed_template.matches("${").count() == 1
            {
                let path = &processed_template[2..processed_template.len() - 1];
                if path.starts_with(".") {
                    format!("${}", path)
                } else {
                    format!("$.{}", path)
                }
            } else {
                processed_template
            };

            // 情况2：以$.开头的完整JSONPath
            if processed_template.starts_with("$.[") {
                (
                    format!("${}", &processed_template[2..]),
                    context.get("__input").unwrap(),
                )
            } else if processed_template.starts_with("$.") {
                (processed_template, context.get("__input").unwrap())
            } else if processed_template == "$" {
                // 情况3：单独的$，返回输入数据
                (processed_template, context.get("__input").unwrap())
            } else if processed_template.starts_with("$[") {
                // 情况4：数组索引
                (
                    format!("${}", &processed_template[1..]),
                    context.get("__input").unwrap(),
                )
            } else if let Some(stripped) = processed_template.strip_prefix("$params") {
                // 情况5：包含$params
                let path = if stripped.is_empty() {
                    "$".to_string()
                } else {
                    format!("${}", stripped)
                };
                (path, context.get("__params").unwrap())
            } else if let Some(stripped) = processed_template.strip_prefix("$meta") {
                // 情况6：包含$meta
                let path = if stripped.is_empty() {
                    "$".to_string()
                } else {
                    format!("${}", stripped)
                };
                (path, context.get("__meta").unwrap())
            } else if let Some(stripped) = processed_template.strip_prefix("$data") {
                // 情况7：包含$data
                let path = if stripped.is_empty() {
                    "$".to_string()
                } else {
                    format!("${}", stripped)
                };
                (path, context.get("__input").unwrap())
            } else if let Some(stripped) = processed_template.strip_prefix("$input") {
                // 情况8：包含$input
                let path = if stripped.is_empty() {
                    "$".to_string()
                } else {
                    format!("${}", stripped)
                };
                (path, context.get("__input").unwrap())
            } else {
                (processed_template, context)
            }
        };

        if path.contains("${") {
            // 情况9：包含变量的模板字符串
            return Self::resolve_template(context, &path);
        }

        Self::resolve_jsonpath(data, &path.to_string())
    }

    /// 解析JSONPath路径
    fn resolve_jsonpath(context: &Value, path: &str) -> Result<Value, String> {
        // 使用jsonpath_rust库查询
        // println!("resolve_jsonpath - path: {}, context: {:?}", path, context);

        // 检查是否以.length结尾
        if path.ends_with(".length") {
            // 去掉.length后缀，获取原始路径
            let base_path = &path[..path.len() - 7]; // 去掉".length"

            match context.query(base_path) {
                Ok(results) => {
                    if !results.is_empty() {
                        let value = &results[0];
                        // 根据值类型获取长度
                        let length = match value {
                            Value::String(s) => s.len() as i64,
                            Value::Array(a) => a.len() as i64,
                            Value::Object(o) => o.len() as i64,
                            _ => return Ok(Value::Null),
                        };
                        Ok(Value::Number(length.into()))
                    } else {
                        Ok(Value::Null)
                    }
                }
                Err(err) => Err(format!("JSONPath解析错误 '{}': {}", base_path, err)),
            }
        } else {
            // 正常的JSONPath查询
            match context.query(path) {
                Ok(results) => {
                    if !results.is_empty() {
                        Ok(results[0].clone())
                    } else {
                        Ok(Value::Null)
                    }
                }
                Err(err) => Err(format!("JSONPath解析错误 '{}': {}", path, err)),
            }
        }
    }

    /// 解析模板字符串，替换其中的变量
    fn resolve_template(context: &Value, template: &str) -> Result<Value, String> {
        let mut result = template.to_string();

        // 使用正则表达式找到所有${...}变量
        let re = Regex::new(r"\$\{([^}]+)\}").map_err(|e| format!("正则表达式错误: {}", e))?;

        // 替换所有变量
        for caps in re.captures_iter(template) {
            let full_match = caps.get(0).unwrap().as_str();
            let path = caps.get(1).unwrap().as_str();
            // 获取值并转换为字符串
            let value = Self::get_value(context, path)?;
            let value_str = Self::value_to_string(&value);

            // 替换模板中的变量
            result = result.replace(full_match, &value_str);
        }

        // 返回字符串结果
        Ok(Value::String(result))
    }

    /// 将Value转换为字符串
    fn value_to_string(value: &Value) -> String {
        match value {
            Value::String(s) => s.clone(),
            Value::Number(n) => n.to_string(),
            Value::Bool(b) => b.to_string(),
            Value::Null => "null".to_string(),
            _ => value.to_string(),
        }
    }
}
