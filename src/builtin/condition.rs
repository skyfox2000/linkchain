//! 条件过滤挂件
//!
//! 基于条件表达式或JSONPath进行数据过滤

use crate::chainware::config::ChainwareConfig;
use crate::chainware::core::Chainware;
use crate::core::{ChainStatus, ChainRequest, ChainResponse};
use crate::types::{error_codes, ErrorResponse};
use crate::utils::json_path::JsonPathTemplate;
use regex::Regex;
use serde_json::Value;

/// 条件过滤挂件
pub struct ConditionChainware {
    name: String,
}

impl Default for ConditionChainware {
    fn default() -> Self {
        Self::new()
    }
}

impl ConditionChainware {
    pub fn new() -> Self {
        Self {
            name: "condition".to_string(),
        }
    }

    /// 检查条件是否满足
    fn check_condition(&self, condition: &str, context: &Value) -> Result<bool, String> {
        // 如果条件为空，则默认为true
        if condition.is_empty() {
            return Ok(true);
        }

        // 先检查是否包含逻辑运算符，如果包含则使用复杂表达式解析
        if condition.contains(" && ") || condition.contains(" || ") {
            return self.parse_complex_condition(condition, context);
        }

        // 简单条件表达式解析
        if let Some(result) = self.parse_simple_condition(condition, context)? {
            return Ok(result);
        }

        Err(format!("无法解析条件表达式: {}", condition))
    }

    /// 解析简单条件表达式
    /// 支持格式：
    /// - 基本比较: $input.field == value, ${input.field} != value, $input > 10
    /// - 字符串操作: String.startsWith($input.field, "prefix"), String.endsWith($input.field, "suffix")
    /// - 类型检查: Chain.isString($input.field), Chain.isNumber($input.field), Chain.isNull($input.field)
    /// - 长度检查: $input.field.length > 5, $input.array.length == 0
    fn parse_simple_condition(
        &self,
        condition: &str,
        context: &Value,
    ) -> Result<Option<bool>, String> {
        // 检查基本比较运算符，按长度排序避免优先级问题
        let basic_operators = [">=", "<=", "==", "!=", ">", "<"];
        for &op in basic_operators.iter() {
            if let Some((left, right)) = condition.split_once(op) {
                let left = left.trim();
                let right = right.trim();

                // 获取左侧变量值
                let left_value = self.resolve_path(left, context)?;

                // 解析右侧值
                let right_value = self.parse_value(right, context)?;

                // 比较值
                return Ok(Some(match op {
                    "==" => self.equals(&left_value, &right_value),
                    "!=" => !self.equals(&left_value, &right_value),
                    ">" => Self::compare(&left_value, &right_value) > 0,
                    "<" => Self::compare(&left_value, &right_value) < 0,
                    ">=" => Self::compare(&left_value, &right_value) >= 0,
                    "<=" => Self::compare(&left_value, &right_value) <= 0,
                    _ => false,
                }));
            }
        }

        // 检查函数式字符串操作符 String.startsWith(field, "value")
        if let Some(caps) = Regex::new(
            r"String\.(startsWith|endsWith|contains|matches)\s*\(\s*([^,]+)\s*,\s*([^)]+)\s*\)",
        )
        .unwrap()
        .captures(condition)
        {
            let operation = caps.get(1).unwrap().as_str();
            let field = caps.get(2).unwrap().as_str().trim();
            let value_str = caps.get(3).unwrap().as_str().trim();

            let field_value = self.resolve_path(field, context)?;
            let field_str = match &field_value {
                Value::String(s) => s.as_str(),
                _ => return Ok(Some(false)),
            };

            let compare_str = if value_str.starts_with('"') && value_str.ends_with('"') {
                value_str[1..value_str.len() - 1].to_string()
            } else {
                match self.resolve_path(value_str, context)? {
                    Value::String(s) => s,
                    _ => return Ok(Some(false)),
                }
            };

            let result = match operation {
                "startsWith" => field_str.starts_with(&compare_str),
                "endsWith" => field_str.ends_with(&compare_str),
                "contains" => field_str.contains(&compare_str),
                "matches" => match Regex::new(&compare_str) {
                    Ok(re) => re.is_match(field_str),
                    Err(_) => return Err(format!("无效的正则表达式: {}", compare_str)),
                },
                _ => false,
            };

            return Ok(Some(result));
        }

        // 检查函数式类型检查操作符 Chain.isString(field)
        if let Some(caps) = Regex::new(r"Chain\.(isString|isNumber|isBoolean|isObject|isArray|isNull|isEmpty)\s*\(\s*([^)]+)\s*\)")
            .unwrap().captures(condition) {
            let operation = caps.get(1).unwrap().as_str();
            let field = caps.get(2).unwrap().as_str().trim();

            let value = self.resolve_path(field, context)?;

            let result = match operation {
                "isString" => value.is_string(),
                "isNumber" => value.is_number(),
                "isBoolean" => value.is_boolean(),
                "isObject" => value.is_object(),
                "isArray" => value.is_array(),
                "isNull" => value.is_null(),
                "isEmpty" => self.is_empty(&value),
                _ => false,
            };

            return Ok(Some(result));
        }

        // 检查长度属性
        if let Some((path, length_condition)) = condition.split_once(".length ") {
            let value = self.resolve_path(path.trim(), context)?;

            // 获取值的长度
            let length = match &value {
                Value::String(s) => s.len() as i64,
                Value::Array(a) => a.len() as i64,
                Value::Object(o) => o.len() as i64,
                _ => -1, // 其他类型没有长度概念
            };

            // 解析长度条件，按长度排序避免优先级问题
            let length_operators = [">=", "<=", "==", "!=", ">", "<"];
            for &op in length_operators.iter() {
                if let Some((_, right)) = length_condition.split_once(op) {
                    let right = right.trim();
                    let right_value = if let Ok(num) = right.parse::<i64>() {
                        num
                    } else {
                        return Err(format!("长度比较需要数字: {}", right));
                    };

                    // 比较长度
                    return Ok(Some(match op {
                        "==" => length == right_value,
                        "!=" => length != right_value,
                        ">" => length > right_value,
                        "<" => length < right_value,
                        ">=" => length >= right_value,
                        "<=" => length <= right_value,
                        _ => false,
                    }));
                }
            }
        }

        // 如果是单独的路径，检查它是否为truthy值
        if !condition.contains(' ') {
            let value = self.resolve_path(condition.trim(), context)?;
            return Ok(Some(self.is_truthy(&value)));
        }

        // 不是简单条件表达式
        Ok(None)
    }

    /// 解析复杂条件表达式
    /// 支持格式：$input.field1 == value1 && $input.field2 > value2 || String.startsWith($.input.field3, "prefix")
    fn parse_complex_condition(&self, condition: &str, context: &Value) -> Result<bool, String> {
        // 拆分OR条件
        let or_parts: Vec<&str> = condition.split(" || ").collect();

        // 对每个OR部分进行AND条件的检查
        for or_part in or_parts {
            let and_parts: Vec<&str> = or_part.split(" && ").collect();

            // 所有AND条件都满足时，返回true
            let mut all_and_true = true;
            for and_part in and_parts {
                let and_condition = and_part.trim();

                if let Some(result) = self.parse_simple_condition(and_condition, context)? {
                    if !result {
                        all_and_true = false;
                        break;
                    }
                } else {
                    return Err(format!("无法解析条件表达式部分: {}", and_condition));
                }
            }

            // 任何一个OR条件满足时，返回true
            if all_and_true {
                return Ok(true);
            }
        }

        // 所有条件都不满足
        Ok(false)
    }

    /// 解析路径表达式，从上下文中获取值
    /// 支持格式：
    /// - JSONPath: $（整个上下文）, $input（输入数据）, $input.data[0].field 等
    /// - 变量引用: ${input}, ${input.data[0].field} 等
    fn resolve_path(&self, path: &str, context: &Value) -> Result<Value, String> {
        let trimmed_path = path.trim();

        // 处理字面量值
        if !trimmed_path.starts_with("$")
            && !trimmed_path
                .chars()
                .next()
                .is_some_and(|c| c.is_alphabetic())
        {
            return self.parse_literal(trimmed_path);
        }

        // 检查是否以字母开头的简单引用（隐式引用）
        if trimmed_path
            .chars()
            .next()
            .is_some_and(|c| c.is_alphabetic())
        {
            // 默认将简单引用视为input字段下的路径
            let template = format!("$input.{}", trimmed_path);
            return JsonPathTemplate::get_value(context, &template);
        }

        // 使用统一的JsonPathTemplate工具处理所有路径格式
        JsonPathTemplate::get_value(context, trimmed_path)
    }

    /// 解析字面量值
    fn parse_literal(&self, literal: &str) -> Result<Value, String> {
        if literal.starts_with('"') && literal.ends_with('"') {
            // 字符串字面量
            return Ok(Value::String(literal[1..literal.len() - 1].to_string()));
        } else if let Ok(num) = literal.parse::<i64>() {
            // 整数字面量
            return Ok(Value::Number(num.into()));
        } else if let Ok(num) = literal.parse::<f64>() {
            // 浮点数字面量
            if let Some(num) = serde_json::Number::from_f64(num) {
                return Ok(Value::Number(num));
            }
        } else if literal == "true" {
            // 布尔值true
            return Ok(Value::Bool(true));
        } else if literal == "false" {
            // 布尔值false
            return Ok(Value::Bool(false));
        } else if literal == "null" {
            // null值
            return Ok(Value::Null);
        }

        // 不是有效的字面量
        Err(format!("无效的字面量值: {}", literal))
    }

    /// 解析值（可能是字面量或路径）
    fn parse_value(&self, value: &str, context: &Value) -> Result<Value, String> {
        let trimmed_value = value.trim();
        
        // 先尝试解析为字面量
        if let Ok(literal_value) = self.parse_literal(trimmed_value) {
            return Ok(literal_value);
        }
        
        // 如果是路径格式，直接解析
        if trimmed_value.starts_with("$") || trimmed_value.starts_with("${") {
            return self.resolve_path(trimmed_value, context);
        }
        
        // 最后尝试作为字段路径
        let path = format!("$input.{}", trimmed_value);
        JsonPathTemplate::get_value(context, &path)
    }

    /// 比较两个值是否相等
    fn equals(&self, a: &Value, b: &Value) -> bool {
        match (a, b) {
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Number(a), Value::Number(b)) => {
                // 数字比较，考虑整数和浮点数
                if let (Some(a_i64), Some(b_i64)) = (a.as_i64(), b.as_i64()) {
                    a_i64 == b_i64
                } else if let (Some(a_f64), Some(b_f64)) = (a.as_f64(), b.as_f64()) {
                    (a_f64 - b_f64).abs() < f64::EPSILON
                } else {
                    false
                }
            }
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::Null, Value::Null) => true,
            // 数组和对象使用默认比较逻辑
            (Value::Array(a), Value::Array(b)) => a == b,
            (Value::Object(a), Value::Object(b)) => a == b,
            // 特殊情况：数字和字符串比较
            (Value::Number(n), Value::String(s)) | (Value::String(s), Value::Number(n)) => {
                if let Ok(num) = s.parse::<f64>() {
                    if let Some(n_f64) = n.as_f64() {
                        return (n_f64 - num).abs() < f64::EPSILON;
                    }
                }
                false
            }
            // 特殊情况：布尔值和字符串比较
            (Value::Bool(b), Value::String(s)) | (Value::String(s), Value::Bool(b)) => {
                let s = s.to_lowercase();
                (*b && (s == "true" || s == "yes" || s == "1"))
                    || (!*b && (s == "false" || s == "no" || s == "0"))
            }
            _ => false, // 不同类型的值不相等
        }
    }

    /// 比较两个值的大小（返回：1=大于，0=等于，-1=小于）
    fn compare(a: &Value, b: &Value) -> i8 {
        match (a, b) {
            (Value::String(a), Value::String(b)) => match a.partial_cmp(b) {
                Some(std::cmp::Ordering::Greater) => 1,
                Some(std::cmp::Ordering::Less) => -1,
                _ => 0,
            },
            (Value::Number(a), Value::Number(b)) => {
                // 数字比较，优先使用整数比较
                if let (Some(a_i64), Some(b_i64)) = (a.as_i64(), b.as_i64()) {
                    match a_i64.cmp(&b_i64) {
                        std::cmp::Ordering::Greater => 1,
                        std::cmp::Ordering::Less => -1,
                        std::cmp::Ordering::Equal => 0,
                    }
                } else if let (Some(a_f64), Some(b_f64)) = (a.as_f64(), b.as_f64()) {
                    match a_f64.partial_cmp(&b_f64) {
                        Some(std::cmp::Ordering::Greater) => 1,
                        Some(std::cmp::Ordering::Less) => -1,
                        _ => 0,
                    }
                } else {
                    0 // 不可比较的数字
                }
            }
            // 字符串和数字比较
            (Value::String(s), Value::Number(n)) => {
                if let Ok(num) = s.parse::<f64>() {
                    if let Some(n_f64) = n.as_f64() {
                        match num.partial_cmp(&n_f64) {
                            Some(std::cmp::Ordering::Greater) => 1,
                            Some(std::cmp::Ordering::Less) => -1,
                            _ => 0,
                        }
                    } else {
                        0
                    }
                } else {
                    0
                }
            }
            (Value::Number(n), Value::String(s)) => {
                // 反转比较结果
                -Self::compare(&Value::String(s.clone()), &Value::Number(n.clone()))
            }
            _ => 0, // 其他类型不支持大小比较
        }
    }

    /// 检查值是否为空
    fn is_empty(&self, value: &Value) -> bool {
        match value {
            Value::String(s) => s.is_empty(),
            Value::Array(a) => a.is_empty(),
            Value::Object(o) => o.is_empty(),
            Value::Null => true,
            _ => false,
        }
    }

    /// 检查值是否为truthy（在条件判断中视为true）
    fn is_truthy(&self, value: &Value) -> bool {
        match value {
            Value::Null => false,
            Value::Bool(b) => *b,
            Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    i != 0
                } else if let Some(f) = n.as_f64() {
                    f != 0.0
                } else {
                    false
                }
            }
            Value::String(s) => !s.is_empty() && s != "false" && s != "0",
            Value::Array(a) => !a.is_empty(),
            Value::Object(o) => !o.is_empty(),
        }
    }
}

impl Chainware for ConditionChainware {
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

        // 获取条件表达式
        let condition = match config.and_then(|cfg| {
            cfg.config
                .get("expression")
                .or_else(|| cfg.config.get("condition"))
        }) {
            Some(Value::String(cond)) => cond,
            Some(_) => {
                response.status = ChainStatus::Error;
                response.data = Some(
                    ErrorResponse::new(
                        error_codes::CONFIG_ERROR,
                        "配置中的expression必须是字符串类型".to_string(),
                        None,
                    )
                    .to_json(),
                );
                return None;
            }
            None => {
                response.status = ChainStatus::Error;
                response.data = Some(
                    ErrorResponse::new(error_codes::CONFIG_ERROR, "缺少条件配置".to_string(), None)
                        .to_json(),
                );
                return None;
            }
        };

        // 检查条件
        match self.check_condition(condition, &context) {
            Ok(true) => {
                // 条件通过，继续执行
                Some(input)
            }
            Ok(false) => {
                // 条件不通过，拒绝执行
                response.status = ChainStatus::Reject;
                response.data = Some(
                    ErrorResponse::new(
                        error_codes::CONDITION_NOT_MET,
                        format!("条件检查未通过: {}", condition),
                        None,
                    )
                    .to_json(),
                );
                None
            }
            Err(err) => {
                // 检查出错
                response.status = ChainStatus::Error;
                response.data = Some(
                    ErrorResponse::new(
                        error_codes::INTERNAL_ERROR,
                        format!("条件检查失败: {}, {}", condition, err),
                        Some(input),
                    )
                    .to_json(),
                );
                None
            }
        }
    }
}
