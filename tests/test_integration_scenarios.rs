//! 集成场景测试
//!
//! 测试多个挂件组合使用的场景

use linkchain::chain::executor::ChainExecutor;
use linkchain::core::{ChainRequest, ChainStatus};
use serde_json::json;
use std::collections::HashMap;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_authentication_and_logging() {
        // 创建执行器
        let mut executor = ChainExecutor::new();
        
        // 添加条件判断挂件 - 验证用户年龄
        let mut age_config = HashMap::new();
        age_config.insert("expression".to_string(), json!("$.age >= 18"));
        executor = executor.add_chainwares(json!([{
            "name": "condition",
            "config": age_config
        }])).unwrap();

        // 添加日志记录挂件
        let mut logger_config = HashMap::new();
        logger_config.insert("template".to_string(), json!("用户 ${$.name} 已验证，年龄: ${$.age}"));
        executor = executor.add_chainwares(json!([{
            "name": "logger",
            "config": logger_config
        }])).unwrap();

        // 创建请求上下文
        let input_data = json!({"name": "张三", "age": 25, "city": "北京"});
        let meta = HashMap::new();
        let context = ChainRequest::new(input_data, meta);

        // 执行链
        let response = executor.execute(context);

        // 检查执行结果
        assert_eq!(response.status, ChainStatus::Completed);
        assert!(response.data.is_some());
    }

    #[test]
    fn test_complex_data_processing_pipeline() {
        // 创建执行器
        let mut executor = ChainExecutor::new();
        
        // 添加IP白名单挂件
        let mut ip_whitelist_config = HashMap::new();
        ip_whitelist_config.insert("ip_list".to_string(), json!("192.168.1.0/24,10.0.0.0/8"));
        executor = executor.add_chainwares(json!([{
            "name": "ip_whitelist",
            "config": ip_whitelist_config
        }])).unwrap();

        // 添加数据提取挂件 - 提取用户基本信息
        let mut extract_config = HashMap::new();
        extract_config.insert("mapping".to_string(), json!({
            "user_name": "$.name",
            "user_age": "$.age",
            "user_role": "$.role"
        }));
        executor = executor.add_chainwares(json!([{
            "name": "extract_map",
            "config": extract_config
        }])).unwrap();

        // 添加条件判断挂件 - 验证权限
        let mut permission_config = HashMap::new();
        permission_config.insert("expression".to_string(), json!("$.user_role == 'admin' && $.user_age >= 18"));
        executor = executor.add_chainwares(json!([{
            "name": "condition",
            "config": permission_config
        }])).unwrap();

        // 添加数据合并挂件
        let mut merge_config = HashMap::new();
        merge_config.insert("data_path".to_string(), json!("$meta"));
        executor = executor.add_chainwares(json!([{
            "name": "merge",
            "config": merge_config
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

        // 检查执行结果
        assert_eq!(response.status, ChainStatus::Completed);
        assert!(response.data.is_some());
        
        let result_data = response.data.unwrap();
        assert!(result_data.get("timestamp").is_some());
        assert!(result_data.get("session_id").is_some());
    }

    #[test]
    fn test_reject_flow_with_blacklist() {
        // 创建执行器
        let mut executor = ChainExecutor::new();
        
        // 添加IP黑名单挂件
        let mut ip_blacklist_config = HashMap::new();
        ip_blacklist_config.insert("ip_list".to_string(), json!("192.168.1.100"));
        executor = executor.add_chainwares(json!([{
            "name": "ip_blacklist",
            "config": ip_blacklist_config
        }])).unwrap();

        // 添加条件判断挂件
        let mut condition_config = HashMap::new();
        condition_config.insert("expression".to_string(), json!("$.score > 80"));
        executor = executor.add_chainwares(json!([{
            "name": "condition",
            "config": condition_config
        }])).unwrap();

        // 创建请求上下文 (使用黑名单中的IP)
        let input_data = json!({"name": "用户", "score": 95});
        let mut meta = HashMap::new();
        meta.insert("ip_address".to_string(), json!("192.168.1.100"));
        let context = ChainRequest::new(input_data, meta);

        // 执行链
        let response = executor.execute(context);

        // 检查执行结果应该是Reject状态
        assert_eq!(response.status, ChainStatus::Reject);
    }
}