// 通用测试辅助模块
//
// 提供统一的测试方法和辅助函数

use linkchain::chain::executor::ChainExecutor;
use linkchain::chainware::config::ChainwareConfig;
use linkchain::core::{ChainStatus, ChainRequest};
use serde_json::{json, Value};
use std::collections::HashMap;

/// 测试用例类型定义
pub type TestCase = (
    u32,                                        // test_id
    &'static str,                               // test_name  
    Value,                                      // input_data
    Vec<(&'static str, HashMap<String, Value>)>, // chainware_configs
    ChainStatus,                            // expected_status
    Option<&'static str>,                       // ip
    Option<Value>,                              // expected_result
);

/// 创建配置辅助函数
pub fn create_config(config_map: HashMap<String, Value>) -> ChainwareConfig {
    ChainwareConfig { config: config_map }
}

/// 创建带IP的测试上下文
pub fn create_context_with_ip(data: Value, ip: Option<&str>) -> ChainRequest {
    let mut meta = HashMap::new();
    if let Some(ip_addr) = ip {
        meta.insert("ip_address".to_string(), json!(ip_addr));
    }
    let context = ChainRequest::new(data, meta);
    context
}

/// 统一测试执行器
pub fn run_test(
    test_id: u32,
    test_name: &str,
    input_data: Value,
    chainware_configs: Vec<(&str, HashMap<String, Value>)>,
    expected_status: ChainStatus,
    ip: Option<&str>,
    expected_result: Option<Value>,
) -> bool {
    println!("📋 测试#{}: {}", test_id, test_name);
    println!("├─ 输入数据: {}", input_data);
    if let Some(ip_addr) = ip {
        println!("├─ IP地址: {}", ip_addr);
    }

    // 构建执行器链
    let mut executor = ChainExecutor::new();
    for (name, config_map) in &chainware_configs {
        let _config = if config_map.is_empty() {
            None
        } else {
            Some(create_config(config_map.clone()))
        };
        executor = executor.add_chainwares(json!([{
            "name": name,
            "config": config_map
        }])).unwrap();
        let config_desc = if config_map.is_empty() {
            "无配置".to_string()
        } else {
            format!("{:?}", config_map)
        };
        println!("├─ 挂件: {} - {}", name, config_desc);
    }

    // 执行测试
    let context = create_context_with_ip(input_data, ip);
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
    println!("{}", "-".repeat(80));

    success
}

/// 执行测试用例并统计结果
pub fn run_test_cases(
    test_cases: Vec<TestCase>,
    test_group_name: &str,
    min_success_rate: f32,
) {
    println!("🧪 {}", test_group_name);
    println!("{}", "=".repeat(80));
    
    let mut passed = 0;
    let total = test_cases.len();
    
    for (test_id, test_name, input_data, configs, expected_status, ip, expected_result) in test_cases {
        if run_test(test_id, test_name, input_data, configs, expected_status, ip, expected_result) {
            passed += 1;
        }
    }
    
    println!("📊 {} 测试统计:", test_group_name);
    println!("• 通过: {} ✅", passed);
    println!("• 失败: {} ❌", total - passed);
    println!("• 成功率: {:.1}%", (passed as f32 / total as f32) * 100.0);

    let expected_passed = (total as f32 * min_success_rate).ceil() as usize;
    assert!(
        passed >= expected_passed,
        "{} 测试成功率应该达到 {:.1}%，期望至少通过 {} 个，实际通过 {} 个",
        test_group_name,
        min_success_rate * 100.0,
        expected_passed,
        passed
    );
} 