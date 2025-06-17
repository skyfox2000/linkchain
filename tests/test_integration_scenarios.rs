//! 集成场景测试
//!
//! 使用ChainExecutor统一管理测试复杂业务场景

use linkchain::chain::executor::ChainExecutor;
use linkchain::chainware::config::ChainwareConfig;
use linkchain::core::{ExecutionStatus, RequestContext};
use serde_json::{json, Value};
use std::collections::HashMap;

fn create_config(config_map: HashMap<String, Value>) -> ChainwareConfig {
    ChainwareConfig { config: config_map }
}

fn run_test(
    test_id: u32,
    test_name: &str,
    input_data: Value,
    chainware_configs: Vec<(&str, HashMap<String, Value>)>,
    expected_status: ExecutionStatus,
    ip: Option<&str>,
    expected_result: Option<Value>,
) -> bool {
    println!("📋 测试#{}: {}", test_id, test_name);
    println!("├─ 输入数据: {}", input_data);
    if let Some(ip_addr) = ip {
        println!("├─ 客户端IP: {}", ip_addr);
    }

    let mut executor = ChainExecutor::new();
    for (name, config_map) in &chainware_configs {
        let config = if config_map.is_empty() {
            None
        } else {
            Some(create_config(config_map.clone()))
        };
        executor = executor.add_chainware(
            name,
            None::<
                fn(
                    &RequestContext,
                    &mut linkchain::core::ResponseContext,
                    Option<Value>,
                    Option<&ChainwareConfig>,
                ) -> Option<Value>,
            >,
            config,
        );
        let config_desc = if config_map.is_empty() {
            "无配置".to_string()
        } else {
            format!("{:?}", config_map)
        };
        println!("├─ 挂件: {} - {}", name, config_desc);
    }

    let mut context = RequestContext::new(input_data);
    if let Some(ip_addr) = ip {
        context
            .meta
            .insert("ip_address".to_string(), json!(ip_addr));
    }

    let response = executor.execute(context);
    let actual_data = response.data.clone();

    println!("├─ 期望状态: {:?}", expected_status);
    println!("├─ 实际状态: {:?}", response.status);
    println!(
        "├─ 期望数据: {}",
        expected_result.clone().unwrap_or(Value::Null)
    );
    println!(
        "├─ 实际数据: {}",
        actual_data.clone().unwrap_or(Value::Null)
    );

    let status_success = response.status == expected_status;
    let data_success = if let Some(expected) = &expected_result {
        actual_data.as_ref().map(|d| d.to_string()) == Some(expected.to_string())
    } else {
        true
    };

    let success = status_success && data_success;

    if !data_success && expected_result.is_some() {
        println!("├─ 数据不匹配!");
    }

    println!(
        "└─ 结果: {} {}",
        if success { "✅" } else { "❌" },
        if success { "成功" } else { "失败" }
    );
    println!("{}", "-".repeat(50));

    success
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_registration() {
        println!("🧪 集成场景：用户注册流程");

        let test_cases = vec![(
            "用户注册流程",
            json!({
                "raw_data": "username=newuser,email=user@example.com,password=secret123"
            }),
            vec![
                ("json_extract", {
                    let mut config = HashMap::new();
                    config.insert("pattern".to_string(), json!("$.raw_data"));
                    config
                }),
                ("regexp_extract", {
                    let mut config = HashMap::new();
                    config.insert(
                        "pattern".to_string(),
                        json!(r"username=([^,]+),email=([^,]+),password=([^,]+)"),
                    );
                    config
                }),
                ("extract_map", {
                    let mut config = HashMap::new();
                    config.insert(
                        "mapping".to_string(),
                        json!({
                            "username": "$input[0]",
                            "email": "$.[1]"
                        }),
                    );
                    config
                }),
                ("logger", {
                    let mut config = HashMap::new();
                    config.insert("level".to_string(), json!("INFO"));
                    config.insert("message".to_string(), json!("用户注册完成"));
                    config
                }),
            ],
            ExecutionStatus::Completed,
            Some("192.168.1.100"),
            Some(json!({"username": "newuser", "email": "user@example.com"})),
        )];

        let mut passed = 0;
        for (test_name, input_data, configs, expected_status, ip, expected_result) in test_cases {
            if run_test(
                1,
                test_name,
                input_data,
                configs,
                expected_status,
                ip,
                expected_result,
            ) {
                passed += 1;
            }
        }

        println!("用户注册流程测试通过: {}/1", passed);
        assert!(passed >= 1);
    }

    #[test]
    fn test_security_check() {
        println!("🧪 集成场景：安全检查流程");

        let test_cases = vec![
            (
                "安全用户访问",
                json!({"user": "admin", "action": "login"}),
                vec![
                    ("ip_whitelist", {
                        let mut config = HashMap::new();
                        config.insert("ip_list".to_string(), json!("192.168.1.0/24"));
                        config
                    }),
                    ("condition", {
                        let mut config = HashMap::new();
                        config.insert("condition".to_string(), json!("$.user == \"admin\""));
                        config
                    }),
                    ("logger", {
                        let mut config = HashMap::new();
                        config.insert("level".to_string(), json!("INFO"));
                        config.insert("message".to_string(), json!("管理员访问"));
                        config
                    }),
                ],
                ExecutionStatus::Completed,
                Some("192.168.1.10"),
                Some(json!({"user": "admin", "action": "login"})),
            ),
            (
                "IP黑名单访问",
                json!({"user": "hacker", "action": "attack"}),
                vec![
                    ("ip_blacklist", {
                        let mut config = HashMap::new();
                        config.insert("ip_list".to_string(), json!("203.0.113.0/24"));
                        config
                    }),
                    ("logger", {
                        let mut config = HashMap::new();
                        config.insert("level".to_string(), json!("WARN"));
                        config.insert("message".to_string(), json!("恶意IP访问"));
                        config
                    }),
                ],
                ExecutionStatus::Reject,
                Some("203.0.113.100"),
                Some(
                    json!({"errno":403,"msg":"IP地址 203.0.113.100 在黑名单中"}),
                ),
            ),
        ];

        let mut passed = 0;
        for (test_name, input_data, configs, expected_status, ip, expected_result) in test_cases {
            if run_test(
                2,
                test_name,
                input_data,
                configs,
                expected_status,
                ip,
                expected_result,
            ) {
                passed += 1;
            }
        }

        println!("安全检查流程测试通过: {}/2", passed);
        assert!(passed >= 1);
    }

    #[test]
    fn test_data_pipeline() {
        println!("🧪 集成场景：数据处理管道");

        let test_cases = vec![(
            "日志数据处理",
            json!({
                "log_entry": "2024-01-01 10:30:45 [INFO] User admin login success"
            }),
            vec![
                ("json_extract", {
                    let mut config = HashMap::new();
                    config.insert("pattern".to_string(), json!("$.log_entry"));
                    config
                }),
                ("regexp_extract", {
                    let mut config = HashMap::new();
                    config.insert("pattern".to_string(), json!(r"(\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2}) \[(\w+)\] User (\w+) (\w+) (\w+)"));
                    config
                }),
                ("extract_map", {
                    let mut config = HashMap::new();
                    config.insert(
                        "mapping".to_string(),
                        json!({
                            "timestamp": "$input[0]",
                            "level": "$[1]",
                            "username": "$[2]"
                        }),
                    );
                    config
                }),
                ("logger", {
                    let mut config = HashMap::new();
                    config.insert("level".to_string(), json!("INFO"));
                    config.insert("message".to_string(), json!("日志处理完成"));
                    config
                }),
            ],
            ExecutionStatus::Completed,
            None,
            Some(json!({"timestamp": "2024-01-01 10:30:45", "level": "INFO", "username": "admin"})),
        )];

        let mut passed = 0;
        for (test_name, input_data, configs, expected_status, ip, expected_result) in test_cases {
            if run_test(
                3,
                test_name,
                input_data,
                configs,
                expected_status,
                ip,
                expected_result,
            ) {
                passed += 1;
            }
        }

        println!("数据处理管道测试通过: {}/1", passed);
        assert!(passed >= 1);
    }
}
