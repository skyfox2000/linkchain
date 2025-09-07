use linkchain::chain::executor::ChainExecutor;
use linkchain::core::{ChainRequest, ChainStatus};
use serde_json::json;
use std::collections::HashMap;

fn main() {
    // 创建执行器
    let mut executor = ChainExecutor::new();
    
    // 添加IP白名单挂件
    let mut ip_whitelist_config = HashMap::new();
    ip_whitelist_config.insert("ip_list".to_string(), json!("192.168.1.0/24,10.0.0.0/8"));
    executor = executor.add_chainwares(json!([{
        "name": "ip_whitelist",
        "config": ip_whitelist_config
    }])).unwrap();

    // 创建请求上下文
    let input_data = json!({
        "name": "管理员",
        "age": 30,
        "role": "admin",
        "permissions": ["read", "write", "delete"]
    });
    
    let mut meta = HashMap::new();
    meta.insert("ip_address".to_string(), json!("192.168.1.100"));
    meta.insert("timestamp".to_string(), json!("2024-01-01T10:00:00Z"));
    meta.insert("session_id".to_string(), json!("session_12345"));
    let context = ChainRequest::new(input_data, meta);

    // 执行链
    let response = executor.execute(context);

    println!("Response status: {:?}", response.status);
    println!("Response data: {:?}", response.data);
}