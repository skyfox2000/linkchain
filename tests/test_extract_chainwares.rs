//! 提取类挂件测试
//!
//! 测试 extract_map, extract_json, extract_sql, json_extract, regexp_extract 挂件

include!("common.rs");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_map() {
        let test_cases = vec![
            (
                1,
                "基本字段提取",
                json!({"name": "张三", "age": 25, "city": "北京"}),
                vec![("extract_map", {
                    let mut config = HashMap::new();
                    config.insert("mapping".to_string(), json!({
                        "user_name": "$.name",
                        "user_age": "$.age",
                        "location": "$.city"
                    }));
                    config
                })],
                ChainStatus::Completed,
                None,
                Some(json!({
                    "user_name": "张三",
                    "user_age": 25,
                    "location": "北京"
                })),
            ),
            (
                2,
                "嵌套对象提取",
                json!({"user": {"profile": {"name": "李四", "age": 30}}, "status": "active"}),
                vec![("extract_map", {
                    let mut config = HashMap::new();
                    config.insert("mapping".to_string(), json!({
                        "username": "$.user.profile.name",
                        "user_age": "$.user.profile.age",
                        "account_status": "$.status"
                    }));
                    config
                })],
                ChainStatus::Completed,
                None,
                Some(json!({
                    "username": "李四",
                    "user_age": 30,
                    "account_status": "active"
                })),
            ),
            (
                3,
                "字面量值混合",
                json!({"name": "王五"}),
                vec![("extract_map", {
                    let mut config = HashMap::new();
                    config.insert("mapping".to_string(), json!({
                        "username": "$.name",
                        "role": "default_user",
                        "version": 1.0
                    }));
                    config
                })],
                ChainStatus::Completed,
                None,
                Some(json!({
                    "username": "王五",
                    "role": "default_user",
                    "version": 1.0
                })),
            ),
            (
                4,
                "数组元素提取",
                json!({"items": [{"name": "商品1", "price": 100}, {"name": "商品2", "price": 200}]}),
                vec![("extract_map", {
                    let mut config = HashMap::new();
                    config.insert("mapping".to_string(), json!({
                        "first_item": "$.items[0].name",
                        "first_price": "$.items[0].price",
                        "total_items": "$.items.length"
                    }));
                    config
                })],
                ChainStatus::Completed,
                None,
                Some(json!({
                    "first_item": "商品1",
                    "first_price": 100,
                    "total_items": 2
                })),
            ),
            (
                5,
                "模板字符串提取",
                json!({"first_name": "张", "last_name": "三", "city": "北京", "age": 25}),
                vec![("extract_map", {
                    let mut config = HashMap::new();
                    config.insert("mapping".to_string(), json!({
                        "full_name": "${$.first_name}${$.last_name}",
                        "summary": "用户 ${$.first_name}${$.last_name} 来自 ${$.city}, 年龄 ${$.age} 岁"
                    }));
                    config
                })],
                ChainStatus::Completed,
                None,
                Some(json!({
                    "full_name": "张三",
                    "summary": "用户 张三 来自 北京, 年龄 25 岁"
                })),
            ),
        ];

        run_test_cases(test_cases, "extract_map挂件测试", 0.8);
    }

    #[test]
    fn test_extract_json() {
        let test_cases = vec![
            (
                1,
                "从文本中提取JSON对象",
                json!("用户信息：{\"name\":\"张三\",\"age\":25} 处理完成"),
                vec![("extract_json", HashMap::new())],
                ChainStatus::Completed,
                None,
                Some(json!({"name":"张三","age":25})),
            ),
            (
                2,
                "提取纯JSON字符串",
                json!("{\"status\":\"success\",\"data\":[1,2,3]}"),
                vec![("extract_json", HashMap::new())],
                ChainStatus::Completed,
                None,
                Some(json!({"status":"success","data":[1,2,3]})),
            ),
            (
                3,
                "提取复杂JSON结构",
                json!("响应数据: {\"user\":{\"id\":123,\"profile\":{\"name\":\"李四\",\"email\":\"li@test.com\"}},\"status\":\"ok\"}"),
                vec![("extract_json", HashMap::new())],
                ChainStatus::Completed,
                None,
                Some(json!({"user":{"id":123,"profile":{"name":"李四","email":"li@test.com"}},"status":"ok"})),
            ),
            (
                4,
                "提取数组JSON",
                json!("商品列表: [{\"id\":1,\"name\":\"商品A\"},{\"id\":2,\"name\":\"商品B\"}] 共2项"),
                vec![("extract_json", HashMap::new())],
                ChainStatus::Completed,
                None,
                Some(json!([{"id":1,"name":"商品A"},{"id":2,"name":"商品B"}])),
            ),
            (
                5,
                "无JSON内容时的错误处理",
                json!("这里没有JSON数据"),
                vec![("extract_json", HashMap::new())],
                ChainStatus::Completed,
                None,
                Some(json!(Value::Null)),
            ),
        ];

        run_test_cases(test_cases, "extract_json挂件测试", 0.8);
    }

    #[test]
    fn test_extract_sql() {
        let test_cases = vec![
            (
                1,
                "提取Markdown SQL代码块",
                json!("查询语句如下：\n```sql\nSELECT * FROM users WHERE age > 18;\n```\n执行结果..."),
                vec![("extract_sql", HashMap::new())],
                ChainStatus::Completed,
                None,
                Some(json!("SELECT * FROM users WHERE age > 18;")),
            ),
            (
                2,
                "提取直接SQL语句",
                json!("INSERT INTO users (name, age) VALUES ('张三', 25);"),
                vec![("extract_sql", HashMap::new())],
                ChainStatus::Completed,
                None,
                Some(json!("INSERT INTO users (name, age) VALUES ('张三', 25);")),
            ),
            (
                3,
                "提取UPDATE语句",
                json!("需要执行 UPDATE products SET price = price * 0.9 WHERE category = 'electronics'; 来更新价格"),
                vec![("extract_sql", HashMap::new())],
                ChainStatus::Completed,
                None,
                Some(json!("UPDATE products SET price = price * 0.9 WHERE category = 'electronics';")),
            ),
            (
                4,
                "提取复杂SQL查询",
                json!("```sql\nSELECT u.name, p.title, COUNT(*) as count\nFROM users u\nJOIN posts p ON u.id = p.user_id\nWHERE u.active = 1\nGROUP BY u.id, p.id\nORDER BY count DESC;\n```"),
                vec![("extract_sql", HashMap::new())],
                ChainStatus::Completed,
                None,
                Some(json!("SELECT u.name, p.title, COUNT(*) as count\nFROM users u\nJOIN posts p ON u.id = p.user_id\nWHERE u.active = 1\nGROUP BY u.id, p.id\nORDER BY count DESC;")),
            ),
            (
                5,
                "无SQL内容时的错误处理",
                json!("这里没有SQL语句"),
                vec![("extract_sql", HashMap::new())],
                ChainStatus::Completed,
                None,
                Some(json!(Value::Null)),
            ),
        ];

        run_test_cases(test_cases, "extract_sql挂件测试", 0.8);
    }

    #[test]
    fn test_json_extract() {
        let test_cases = vec![
            (
                1,
                "简单路径提取",
                json!({"user": {"name": "张三", "age": 25}}),
                vec![("json_extract", {
                    let mut config = HashMap::new();
                    config.insert("pattern".to_string(), json!("$.user.name"));
                    config
                })],
                ChainStatus::Completed,
                None,
                Some(json!("张三")),
            ),
            (
                2,
                "数组元素提取",
                json!({"items": ["apple", "banana", "orange"]}),
                vec![("json_extract", {
                    let mut config = HashMap::new();
                    config.insert("pattern".to_string(), json!("$.items[1]"));
                    config
                })],
                ChainStatus::Completed,
                None,
                Some(json!("banana")),
            ),
            (
                3,
                "深层嵌套提取",
                json!({"data": {"users": [{"profile": {"email": "test@example.com"}}]}}),
                vec![("json_extract", {
                    let mut config = HashMap::new();
                    config.insert("pattern".to_string(), json!("$.data.users[0].profile.email"));
                    config
                })],
                ChainStatus::Completed,
                None,
                Some(json!("test@example.com")),
            ),
            (
                4,
                "数组长度提取",
                json!({"products": [{"id": 1}, {"id": 2}, {"id": 3}]}),
                vec![("json_extract", {
                    let mut config = HashMap::new();
                    config.insert("pattern".to_string(), json!("$.products.length"));
                    config
                })],
                ChainStatus::Completed,
                None,
                Some(json!(3)),
            ),
            (
                5,
                "路径不存在时的错误处理",
                json!({"user": {"name": "张三"}}),
                vec![("json_extract", {
                    let mut config = HashMap::new();
                    config.insert("pattern".to_string(), json!("$.user.nonexistent"));
                    config
                })],
                ChainStatus::Completed,
                None,
                Some(json!(Value::Null)),
            ),
        ];

        run_test_cases(test_cases, "json_extract挂件测试", 0.8);
    }

    #[test]
    fn test_regexp_extract() {
        let test_cases = vec![
            (
                1,
                "提取数字",
                json!("用户ID: 12345"),
                vec![("regexp_extract", {
                    let mut config = HashMap::new();
                    config.insert("pattern".to_string(), json!(r"\d+"));
                    config
                })],
                ChainStatus::Completed,
                None,
                Some(json!("12345")),
            ),
            (
                2,
                "提取邮箱地址",
                json!("联系邮箱：admin@example.com，请及时回复"),
                vec![("regexp_extract", {
                    let mut config = HashMap::new();
                    config.insert("pattern".to_string(), json!(r"[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}"));
                    config
                })],
                ChainStatus::Completed,
                None,
                Some(json!("admin@example.com")),
            ),
            (
                3,
                "提取手机号",
                json!("客服电话：13812345678，工作时间9-18点"),
                vec![("regexp_extract", {
                    let mut config = HashMap::new();
                    config.insert("pattern".to_string(), json!(r"1[3-9]\d{9}"));
                    config
                })],
                ChainStatus::Completed,
                None,
                Some(json!("13812345678")),
            ),
            (
                4,
                "提取日期",
                json!("报告生成时间：2024-01-15 10:30:00"),
                vec![("regexp_extract", {
                    let mut config = HashMap::new();
                    config.insert("pattern".to_string(), json!(r"\d{4}-\d{2}-\d{2}"));
                    config
                })],
                ChainStatus::Completed,
                None,
                Some(json!("2024-01-15")),
            ),
            (
                5,
                "无匹配内容时的错误处理",
                json!("这里没有数字"),
                vec![("regexp_extract", {
                    let mut config = HashMap::new();
                    config.insert("pattern".to_string(), json!(r"\d+"));
                    config
                })],
                ChainStatus::Completed,
                None,
                Some(json!(Value::Null)),
            ),
        ];

        run_test_cases(test_cases, "regexp_extract挂件测试", 0.8);
    }

    #[test]
    fn test_extract_chains() {
        let test_cases = vec![
            (
                1,
                "提取链：JSON提取 + 字段映射",
                json!({"user": {"profile": {"name": "张三", "email": "zhang@test.com", "age": 28}}}),
                vec![
                    ("json_extract", {
                        let mut config = HashMap::new();
                        config.insert("pattern".to_string(), json!("$.user.profile"));
                        config
                    }),
                    ("extract_map", {
                        let mut config = HashMap::new();
                        config.insert("mapping".to_string(), json!({
                            "username": "$.name",
                            "contact_email": "$.email",
                            "user_age": "$.age"
                        }));
                        config
                    }),
                ],
                ChainStatus::Completed,
                None,
                Some(json!({
                    "username": "张三",
                    "contact_email": "zhang@test.com",
                    "user_age": 28
                })),
            ),
            (
                2,
                "正则提取 + 条件验证",
                json!("手机号码：13812345678"),
                vec![
                    ("regexp_extract", {
                        let mut config = HashMap::new();
                        config.insert("pattern".to_string(), json!(r"1[3-9]\d{9}"));
                        config
                    }),
                    ("regexp_condition", {
                        let mut config = HashMap::new();
                        config.insert("pattern".to_string(), json!(r"^1[3-9]\d{9}$"));
                        config
                    }),
                ],
                ChainStatus::Completed,
                None,
                Some(json!("13812345678")),
            ),
        ];

        run_test_cases(test_cases, "提取挂件链式测试", 0.5);
    }
}
