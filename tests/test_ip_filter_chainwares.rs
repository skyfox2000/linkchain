//! IP过滤类挂件测试
//!
//! 测试 ip_blacklist 和 ip_whitelist 挂件

include!("common.rs");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ip_blacklist() {
        let test_cases = vec![
            (
                1,
                "正常IP通过黑名单",
                json!({"user": "test"}),
                vec![("ip_blacklist", {
                    let mut config = HashMap::new();
                    config.insert("ip_list".to_string(), json!("192.168.1.100,10.0.0.50"));
                    config
                })],
                ChainStatus::Completed,
                Some("192.168.1.200"),
                Some(json!({"user": "test"})),
            ),
            (
                2,
                "黑名单IP被拒绝",
                json!({"user": "test"}),
                vec![("ip_blacklist", {
                    let mut config = HashMap::new();
                    config.insert("ip_list".to_string(), json!("192.168.1.100,10.0.0.50"));
                    config
                })],
                ChainStatus::Reject,
                Some("192.168.1.100"),
                Some(json!( {"errno":403,"msg":"IP地址 192.168.1.100 在黑名单中"})),
            ),
            (
                3,
                "CIDR网段黑名单测试",
                json!({"user": "test"}),
                vec![("ip_blacklist", {
                    let mut config = HashMap::new();
                    config.insert("ip_list".to_string(), json!("192.168.0.0/16,10.0.0.0/8"));
                    config
                })],
                ChainStatus::Reject,
                Some("192.168.100.50"),
                Some(json!({"errno": 403, "msg": "IP地址 192.168.100.50 在黑名单中"})),
            ),
            (
                4,
                "非黑名单网段通过",
                json!({"user": "test"}),
                vec![("ip_blacklist", {
                    let mut config = HashMap::new();
                    config.insert("ip_list".to_string(), json!("192.168.0.0/16,10.0.0.0/8"));
                    config
                })],
                ChainStatus::Completed,
                Some("8.8.8.8"),
                Some(json!({"user": "test"})),
            ),
            (
                5,
                "混合IP和网段黑名单",
                json!({"user": "test"}),
                vec![("ip_blacklist", {
                    let mut config = HashMap::new();
                    config.insert("ip_list".to_string(), json!("1.2.3.4,192.168.0.0/24,172.16.0.100"));
                    config
                })],
                ChainStatus::Reject,
                Some("192.168.0.150"),
                Some(json!({"errno": 403, "msg": "IP地址 192.168.0.150 在黑名单中"})),
            ),
        ];

        run_test_cases(test_cases, "IP黑名单挂件测试", 0.8);
    }

    #[test]
    fn test_ip_whitelist() {
        let test_cases = vec![
            (
                1,
                "白名单IP通过",
                json!({"user": "admin"}),
                vec![("ip_whitelist", {
                    let mut config = HashMap::new();
                    config.insert("ip_list".to_string(), json!("127.0.0.1,192.168.1.100"));
                    config
                })],
                ChainStatus::Completed,
                Some("192.168.1.100"),
                Some(json!({"user": "admin"})),
            ),
            (
                2,
                "非白名单IP被拒绝",
                json!({"user": "test"}),
                vec![("ip_whitelist", {
                    let mut config = HashMap::new();
                    config.insert("ip_list".to_string(), json!("127.0.0.1,192.168.1.100"));
                    config
                })],
                ChainStatus::Reject,
                Some("192.168.1.200"),
                Some(json!({"errno": 403, "msg": "IP地址 192.168.1.200 不在白名单中"})),
            ),
            (
                3,
                "白名单网段测试",
                json!({"user": "internal"}),
                vec![("ip_whitelist", {
                    let mut config = HashMap::new();
                    config.insert("ip_list".to_string(), json!("192.168.0.0/16,10.0.0.0/8"));
                    config
                })],
                ChainStatus::Completed,
                Some("192.168.50.100"),
                Some(json!({"user": "internal"})),
            ),
            (
                4,
                "本地环回地址通过",
                json!({"user": "localhost"}),
                vec![("ip_whitelist", {
                    let mut config = HashMap::new();
                    config.insert("ip_list".to_string(), json!("127.0.0.1,::1"));
                    config
                })],
                ChainStatus::Completed,
                Some("127.0.0.1"),
                Some(json!({"user": "localhost"})),
            ),
            (
                5,
                "办公室白名单测试",
                json!({"user": "employee"}),
                vec![("ip_whitelist", {
                    let mut config = HashMap::new();
                    config.insert("ip_list".to_string(), json!("203.0.113.0/24,198.51.100.0/24"));
                    config
                })],
                ChainStatus::Completed,
                Some("203.0.113.100"),
                Some(json!({"user": "employee"})),
            ),
        ];

        run_test_cases(test_cases, "IP白名单挂件测试", 0.8);
    }

    #[test]
    fn test_ip_filter_chains() {
        let test_cases = vec![
            (
                1,
                "白名单 + 黑名单双重过滤",
                json!({"user": "admin", "action": "login"}),
                vec![
                    ("ip_whitelist", {
                        let mut config = HashMap::new();
                        config.insert("ip_list".to_string(), json!("192.168.0.0/16,10.0.0.0/8"));
                        config
                    }),
                    ("ip_blacklist", {
                        let mut config = HashMap::new();
                        config.insert("ip_list".to_string(), json!("192.168.1.100,192.168.1.101"));
                        config
                    }),
                ],
                ChainStatus::Completed,
                Some("192.168.1.200"),
                Some(json!({"user": "admin", "action": "login"})),
            ),
            (
                2,
                "白名单通过但黑名单拒绝",
                json!({"user": "suspicious"}),
                vec![
                    ("ip_whitelist", {
                        let mut config = HashMap::new();
                        config.insert("ip_list".to_string(), json!("192.168.0.0/16"));
                        config
                    }),
                    ("ip_blacklist", {
                        let mut config = HashMap::new();
                        config.insert("ip_list".to_string(), json!("192.168.1.100"));
                        config
                    }),
                ],
                ChainStatus::Reject,
                Some("192.168.1.100"),
                Some(json!({"errno": 403, "msg": "IP地址 192.168.1.100 在黑名单中"})),
            ),
            (
                3,
                "IP过滤 + 条件验证",
                json!({"user": "admin", "role": "administrator"}),
                vec![
                    ("ip_whitelist", {
                        let mut config = HashMap::new();
                        config.insert("ip_list".to_string(), json!("192.168.1.0/24"));
                        config
                    }),
                    ("condition", {
                        let mut config = HashMap::new();
                        config.insert("expression".to_string(), json!("$.role == \"administrator\""));
                        config
                    }),
                ],
                ChainStatus::Completed,
                Some("192.168.1.50"),
                Some(json!({"user": "admin", "role": "administrator"})),
            ),
        ];

        run_test_cases(test_cases, "IP过滤挂件链式测试", 0.6);
    }

    #[test]
    fn test_ip_edge_cases() {
        let test_cases = vec![
            (
                1,
                "IPv6地址测试",
                json!({"user": "test"}),
                vec![("ip_whitelist", {
                    let mut config = HashMap::new();
                    config.insert("ip_list".to_string(), json!("::1,127.0.0.1"));
                    config
                })],
                ChainStatus::Completed,
                Some("::1"),
                Some(json!({"user": "test"})),
            ),
            (
                2,
                "无效IP地址处理",
                json!({"user": "test"}),
                vec![("ip_blacklist", {
                    let mut config = HashMap::new();
                    config.insert("ip_list".to_string(), json!("192.168.1.100"));
                    config
                })],
                ChainStatus::Error,
                Some("invalid.ip.address"),
                Some(json!({"errno": 500, "msg": "IP黑名单检查失败: 无效的IP地址: invalid.ip.address"})),
            ),
            (
                3,
                "空IP配置测试",
                json!({"user": "test"}),
                vec![("ip_whitelist", {
                    let mut config = HashMap::new();
                    config.insert("ip_list".to_string(), json!(""));
                    config
                })],
                ChainStatus::Reject,
                Some("192.168.1.100"),
                Some(json!({"errno": 403, "msg": "IP地址 192.168.1.100 不在白名单中"})),
            ),
        ];

        run_test_cases(test_cases, "IP过滤边界测试", 0.6);
    }
}
