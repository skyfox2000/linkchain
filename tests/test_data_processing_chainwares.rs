//! 数据处理类挂件测试
//!
//! 测试 logger, merge, map_fields 挂件

include!("common.rs");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_logger() {
        let test_cases = vec![
            (
                1,
                "简单日志记录",
                json!({"username": "张三", "action": "login"}),
                vec![("logger", {
                    let mut config = HashMap::new();
                    config.insert("template".to_string(), json!("用户 ${$.username} 执行了 ${$.action} 操作"));
                    config
                })],
                ChainStatus::Completed,
                None,
                Some(json!({"username": "张三", "action": "login"})),
            ),
            (
                2,
                "复杂模板日志",
                json!({"user": {"name": "李四", "role": "admin"}, "timestamp": "2024-01-01T10:00:00Z"}),
                vec![("logger", {
                    let mut config = HashMap::new();
                    config.insert("template".to_string(), json!("管理员 ${$.user.name} (${$.user.role}) 在 ${$.timestamp} 登录系统"));
                    config
                })],
                ChainStatus::Completed,
                None,
                Some(json!({"user": {"name": "李四", "role": "admin"}, "timestamp": "2024-01-01T10:00:00Z"})),
            ),
            (
                3,
                "数组信息日志",
                json!({"customer": {"name": "王五"}, "items": [{"name": "商品1"}, {"name": "商品2"}], "total": 299.99}),
                vec![("logger", {
                    let mut config = HashMap::new();
                    config.insert("template".to_string(), json!("客户 ${$.customer.name} 购买了 ${$.items.length} 件商品，总金额 ${$.total} 元"));
                    config
                })],
                ChainStatus::Completed,
                None,
                Some(json!({"customer": {"name": "王五"}, "items": [{"name": "商品1"}, {"name": "商品2"}], "total": 299.99})),
            ),
        ];

        run_test_cases(test_cases, "logger挂件测试", 0.8);
    }

    #[test]
    fn test_map_fields() {
        let test_cases = vec![
            (
                1,
                "基础字段映射 - 保留原字段",
                json!({"name": "张三", "age": 25, "city": "北京"}),
                vec![("map_fields", {
                    let mut config = HashMap::new();
                    config.insert("mapping".to_string(), json!({
                        "username": "$.name",
                        "user_age": "$.age",
                        "location": "$.city"
                    }));
                    config.insert("overwrite".to_string(), json!(true));
                    config
                })],
                ChainStatus::Completed,
                None,
                Some(json!({
                    "name": "张三",
                    "age": 25, 
                    "city": "北京",
                    "username": "张三",
                    "user_age": 25,
                    "location": "北京"
                })),
            ),
            (
                2,
                "严格映射 - 只保留映射字段",
                json!({"user_id": 123, "email_address": "test@example.com", "permissions": {"role": "admin"}}),
                vec![("map_fields", {
                    let mut config = HashMap::new();
                    config.insert("mapping".to_string(), json!({
                        "id": "$.user_id",
                        "email": "$.email_address", 
                        "role": "$.permissions.role"
                    }));
                    config.insert("overwrite".to_string(), json!(false));
                    config
                })],
                ChainStatus::Completed,
                None,
                Some(json!({
                    "id": 123,
                    "email": "test@example.com",
                    "role": "admin"
                })),
            ),
        ];

        run_test_cases(test_cases, "map_fields挂件测试", 0.8);
    }

    #[test]
    fn test_merge() {
        let test_cases = vec![
            (
                1,
                "基本数据合并",
                json!({"name": "张三", "age": 25}),
                vec![("merge", {
                    let mut config = HashMap::new();
                    config.insert("data_path".to_string(), json!("$params.extra_info"));
                    config
                })],
                ChainStatus::Completed,
                None,
                Some(json!({"name": "张三", "age": 25})), // 注意：实际实现需要params数据
            ),
        ];

        run_test_cases(test_cases, "merge挂件测试", 0.6); // 由于需要params数据，成功率可能较低
    }

    #[test]
    fn test_data_processing_chains() {
        let test_cases = vec![
            (
                1,
                "日志 + 字段映射链",
                json!({"username": "admin", "action": "login", "timestamp": "2024-01-01T10:00:00Z"}),
                vec![
                    ("logger", {
                        let mut config = HashMap::new();
                        config.insert("template".to_string(), json!("用户 ${$.username} 在 ${$.timestamp} 执行了 ${$.action}"));
                        config
                    }),
                    ("map_fields", {
                        let mut config = HashMap::new();
                        config.insert("mapping".to_string(), json!({
                            "user": "$.username",
                            "operation": "$.action",
                            "time": "$.timestamp"
                        }));
                        config.insert("overwrite".to_string(), json!(false));
                        config
                    }),
                ],
                ChainStatus::Completed,
                None,
                Some(json!({
                    "user": "admin",
                    "operation": "login", 
                    "time": "2024-01-01T10:00:00Z"
                })),
            ),
        ];

        run_test_cases(test_cases, "数据处理挂件链式测试", 0.5);
    }
}
