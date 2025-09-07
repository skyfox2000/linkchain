//! 条件类挂件测试
//!
//! 测试 condition 和 regexp_condition 挂件

include!("common.rs");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_condition_basic() {
        let test_cases = vec![
            (
                1,
                "数值相等条件",
                json!({"age": 25}),
                vec![("condition", {
                    let mut config = HashMap::new();
                    config.insert("expression".to_string(), json!("$.age == 25"));
                    config
                })],
                ChainStatus::Completed,
                None,
                Some(json!({"age": 25})),
            ),
            (
                2,
                "字符串相等条件",
                json!({"role": "admin"}),
                vec![("condition", {
                    let mut config = HashMap::new();
                    config.insert("expression".to_string(), json!("$.role == \"admin\""));
                    config
                })],
                ChainStatus::Completed,
                None,
                Some(json!({"role": "admin"})),
            ),
            (
                3,
                "条件不满足被拒绝",
                json!({"status": "inactive"}),
                vec![("condition", {
                    let mut config = HashMap::new();
                    config.insert("expression".to_string(), json!("$.status == \"active\""));
                    config
                })],
                ChainStatus::Reject,
                None,
                Some(json!({"errno": 401, "msg": "条件检查未通过: $.status == \"active\""})),
            ),
            (
                4,
                "数值大小比较",
                json!({"score": 85}),
                vec![("condition", {
                    let mut config = HashMap::new();
                    config.insert("expression".to_string(), json!("$.score >= 80"));
                    config
                })],
                ChainStatus::Completed,
                None,
                Some(json!({"score": 85})),
            ),
            (
                5,
                "逻辑AND条件",
                json!({"age": 25, "verified": true}),
                vec![("condition", {
                    let mut config = HashMap::new();
                    config.insert(
                        "expression".to_string(),
                        json!("$.age >= 18 && $.verified == true"),
                    );
                    config
                })],
                ChainStatus::Completed,
                None,
                Some(json!({"age": 25, "verified": true})),
            ),
        ];

        run_test_cases(test_cases, "条件挂件基础测试", 0.8);
    }

    #[test]
    fn test_condition_advanced() {
        let test_cases = vec![
            (
                1,
                "字符串函数测试 - startsWith",
                json!({"email": "admin@example.com"}),
                vec![("condition", {
                    let mut config = HashMap::new();
                    config.insert(
                        "expression".to_string(),
                        json!("String.startsWith($.email, \"admin@\")"),
                    );
                    config
                })],
                ChainStatus::Completed,
                None,
                Some(json!({"email": "admin@example.com"})),
            ),
            (
                2,
                "字符串函数测试 - endsWith",
                json!({"email": "user@company.com"}),
                vec![("condition", {
                    let mut config = HashMap::new();
                    config.insert(
                        "expression".to_string(),
                        json!("String.endsWith($.email, \"@company.com\")"),
                    );
                    config
                })],
                ChainStatus::Completed,
                None,
                Some(json!({"email": "user@company.com"})),
            ),
            (
                3,
                "类型检查 - isString",
                json!({"name": "张三"}),
                vec![("condition", {
                    let mut config = HashMap::new();
                    config.insert("expression".to_string(), json!("Chain.isString($.name)"));
                    config
                })],
                ChainStatus::Completed,
                None,
                Some(json!({"name": "张三"})),
            ),
            (
                4,
                "类型检查 - isNumber",
                json!({"age": 25}),
                vec![("condition", {
                    let mut config = HashMap::new();
                    config.insert("expression".to_string(), json!("Chain.isNumber($.age)"));
                    config
                })],
                ChainStatus::Completed,
                None,
                Some(json!({"age": 25})),
            ),
            (
                5,
                "长度检查",
                json!({"username": "testuser"}),
                vec![("condition", {
                    let mut config = HashMap::new();
                    config.insert("expression".to_string(), json!("$.username.length >= 6"));
                    config
                })],
                ChainStatus::Completed,
                None,
                Some(json!({"username": "testuser"})),
            ),
        ];

        run_test_cases(test_cases, "条件挂件高级测试", 0.8);
    }

    #[test]
    fn test_regexp_condition() {
        let test_cases = vec![
            (
                1,
                "数字正则匹配",
                json!("12345"),
                vec![("regexp_condition", {
                    let mut config = HashMap::new();
                    config.insert("pattern".to_string(), json!(r"^\d+$"));
                    config
                })],
                ChainStatus::Completed,
                None,
                Some(json!("12345")),
            ),
            (
                2,
                "邮箱格式验证",
                json!("test@example.com"),
                vec![("regexp_condition", {
                    let mut config = HashMap::new();
                    config.insert(
                        "pattern".to_string(),
                        json!(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$"),
                    );
                    config
                })],
                ChainStatus::Completed,
                None,
                Some(json!("test@example.com")),
            ),
            (
                3,
                "手机号验证",
                json!("13812345678"),
                vec![("regexp_condition", {
                    let mut config = HashMap::new();
                    config.insert("pattern".to_string(), json!(r"^1[3-9]\d{9}$"));
                    config
                })],
                ChainStatus::Completed,
                None,
                Some(json!("13812345678")),
            ),
            (
                4,
                "正则不匹配",
                json!("abc123"),
                vec![("regexp_condition", {
                    let mut config = HashMap::new();
                    config.insert("pattern".to_string(), json!(r"^\d+$"));
                    config
                })],
                ChainStatus::Reject,
                None,
                Some(json!({"errno": 401, "msg": "数据不符合正则规则: ^\\d+$"})),
            ),
            (
                5,
                "用户名格式验证",
                json!("test_user123"),
                vec![("regexp_condition", {
                    let mut config = HashMap::new();
                    config.insert("pattern".to_string(), json!(r"^[a-zA-Z0-9_]{3,20}$"));
                    config
                })],
                ChainStatus::Completed,
                None,
                Some(json!("test_user123")),
            ),
        ];

        run_test_cases(test_cases, "正则条件挂件测试", 0.8);
    }

    #[test]
    fn test_condition_strict_equality() {
        let test_cases = vec![
            (
                1,
                "严格相等比较 - 字符串",
                json!({"value": "123"}),
                vec![("condition", {
                    let mut config = HashMap::new();
                    config.insert("expression".to_string(), json!("$.value === \"123\""));
                    config
                })],
                ChainStatus::Completed,
                None,
                Some(json!({"value": "123"})),
            ),
            (
                2,
                "严格相等比较 - 数字",
                json!({"value": 123}),
                vec![("condition", {
                    let mut config = HashMap::new();
                    config.insert("expression".to_string(), json!("$.value === 123"));
                    config
                })],
                ChainStatus::Completed,
                None,
                Some(json!({"value": 123})),
            ),
            (
                3,
                "严格相等比较 - 类型不匹配",
                json!({"value": "123"}),
                vec![("condition", {
                    let mut config = HashMap::new();
                    config.insert("expression".to_string(), json!("$.value === 123"));
                    config
                })],
                ChainStatus::Reject,
                None,
                Some(json!({ "errno": 401, "msg": "条件检查未通过: $.value === 123" })),
            ),
            (
                4,
                "undefined 比较 - 字段不存在",
                json!({}),
                vec![("condition", {
                    let mut config = HashMap::new();
                    config.insert("expression".to_string(), json!("$.nonexistent === undefined"));
                    config
                })],
                ChainStatus::Completed,
                None,
                Some(json!({})),
            ),
            (
                5,
                "undefined 比较 - 字段存在但不匹配",
                json!({"value": "test"}),
                vec![("condition", {
                    let mut config = HashMap::new();
                    config.insert("expression".to_string(), json!("$.value === undefined"));
                    config
                })],
                ChainStatus::Reject,
                None,
                Some(json!({ "errno": 401, "msg": "条件检查未通过: $.value === undefined" })),
            ),
        ];

        run_test_cases(test_cases, "严格相等和 undefined 比较测试", 0.5);
    }

    #[test]
    fn test_condition_chains() {
        let test_cases = vec![
            (
                1,
                "多条件链式验证",
                json!({"username": "admin", "age": 25, "email": "admin@test.com"}),
                vec![
                    ("condition", {
                        let mut config = HashMap::new();
                        config.insert("expression".to_string(), json!("$.age >= 18"));
                        config
                    }),
                    // ("regexp_condition", {
                    //     let mut config = HashMap::new();
                    //     config.insert(
                    //         "pattern".to_string(),
                    //         json!(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$"),
                    //     );
                    //     config
                    // }),
                    ("condition", {
                        let mut config = HashMap::new();
                        config.insert("expression".to_string(), json!("$.username == \"admin\""));
                        config
                    }),
                ],
                ChainStatus::Completed,
                None,
                Some(json!({"username": "admin", "age": 25, "email": "admin@test.com"})),
            ),
            (
                2,
                "条件失败中断链",
                json!({"username": "user", "age": 16}),
                vec![
                    ("condition", {
                        let mut config = HashMap::new();
                        config.insert("expression".to_string(), json!("$.age >= 18"));
                        config
                    }),
                    ("condition", {
                        let mut config = HashMap::new();
                        config.insert("expression".to_string(), json!("$.username == \"admin\""));
                        config
                    }),
                ],
                ChainStatus::Reject,
                None,
                Some(json!({"errno": 401, "msg": "条件检查未通过: $.age >= 18"})),
            ),
        ];

        run_test_cases(test_cases, "条件挂件链式测试", 0.5);
    }
}
