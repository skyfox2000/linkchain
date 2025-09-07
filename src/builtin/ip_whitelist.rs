//! IP白名单过滤挂件
//!
//! 检查IP地址是否在白名单中，如果不在则拒绝执行

use crate::chainware::config::ChainwareConfig;
use crate::chainware::core::Chainware;
use crate::core::{ChainStatus, ChainRequest, ChainResponse};
use crate::types::{error_codes, ErrorResponse};
use crate::utils::ip_utils;
use serde_json::Value;
use std::net::IpAddr;

/// IP白名单过滤挂件
pub struct IpWhitelistChainware {
    name: String,
}

impl Default for IpWhitelistChainware {
    fn default() -> Self {
        Self::new()
    }
}

impl IpWhitelistChainware {
    pub fn new() -> Self {
        Self {
            name: "ip_whitelist".to_string(),
        }
    }

    /// 检查IP是否在白名单中
    fn is_ip_in_whitelist(&self, ip: &str, whitelist: &[String]) -> Result<bool, String> {
        // 解析IP地址
        let target_ip: IpAddr = ip.parse().map_err(|_| format!("无效的IP地址: {}", ip))?;

        // 检查是否在白名单中
        for whitelist_ip in whitelist {
            // 支持单个IP和CIDR格式
            if whitelist_ip.contains('/') {
                // CIDR格式处理
                if self.ip_in_cidr(&target_ip, whitelist_ip)? {
                    return Ok(true);
                }
            } else {
                // 单个IP地址比较
                let white_ip: IpAddr = whitelist_ip
                    .parse()
                    .map_err(|_| format!("白名单中的无效IP地址: {}", whitelist_ip))?;
                if target_ip == white_ip {
                    return Ok(true);
                }
            }
        }

        Ok(false)
    }

    /// 检查IP是否在CIDR网段中
    fn ip_in_cidr(&self, ip: &IpAddr, cidr: &str) -> Result<bool, String> {
        let parts: Vec<&str> = cidr.split('/').collect();
        if parts.len() != 2 {
            return Err(format!("无效的CIDR格式: {}", cidr));
        }

        let network_ip: IpAddr = parts[0]
            .parse()
            .map_err(|_| format!("CIDR中的无效IP地址: {}", parts[0]))?;
        let prefix_len: u8 = parts[1]
            .parse()
            .map_err(|_| format!("CIDR中的无效前缀长度: {}", parts[1]))?;

        match (ip, network_ip) {
            (IpAddr::V4(ip4), IpAddr::V4(net4)) => {
                if prefix_len > 32 {
                    return Err("IPv4前缀长度不能超过32".to_string());
                }
                let mask = if prefix_len == 0 {
                    0
                } else {
                    !0u32 << (32 - prefix_len)
                };
                Ok((u32::from(*ip4) & mask) == (u32::from(net4) & mask))
            }
            (IpAddr::V6(ip6), IpAddr::V6(net6)) => {
                if prefix_len > 128 {
                    return Err("IPv6前缀长度不能超过128".to_string());
                }
                let ip_bytes = ip6.octets();
                let net_bytes = net6.octets();
                let full_bytes = prefix_len / 8;
                let remaining_bits = prefix_len % 8;

                // 比较完整字节
                for i in 0..full_bytes as usize {
                    if ip_bytes[i] != net_bytes[i] {
                        return Ok(false);
                    }
                }

                // 比较剩余位
                if remaining_bits > 0 {
                    let mask = 0xFF << (8 - remaining_bits);
                    let idx = full_bytes as usize;
                    if idx < 16 && (ip_bytes[idx] & mask) != (net_bytes[idx] & mask) {
                        return Ok(false);
                    }
                }

                Ok(true)
            }
            _ => Err("IP地址类型不匹配".to_string()),
        }
    }
}

impl Chainware for IpWhitelistChainware {
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

        // 从配置中获取IP白名单
        let ip_list = match ip_utils::extract_ip_list(config.and_then(|cfg| cfg.config.get("ip_list"))) {
            Ok(list) => list,
            Err(err) => {
                response.status = ChainStatus::Error;
                response.data = Some(
                    ErrorResponse::new(
                        error_codes::CONFIG_ERROR,
                        err,
                        None,
                    )
                    .to_json(),
                );
                return Some(input); // 数据透传
            }
        };

        // 从配置中获取IP地址字段名，默认为"ip_address"
        let ip_key = config
            .and_then(|cfg| cfg.config.get("ip_key"))
            .and_then(|v| v.as_str())
            .unwrap_or("ip_address");

        // 从meta中获取IP地址
        let ip_address = match request.meta.get(ip_key) {
            Some(Value::String(ip)) => ip,
            Some(_) => {
                response.status = ChainStatus::Reject;
                response.data = Some(
                    ErrorResponse::new(
                        error_codes::FORBIDDEN,
                        format!("meta中的{}必须是字符串类型", ip_key),
                        None,
                    )
                    .to_json(),
                );
                return None;
            }
            None => {
                response.status = ChainStatus::Reject;
                response.data = Some(
                    ErrorResponse::new(
                        error_codes::FORBIDDEN,
                        format!("meta中缺少{}", ip_key),
                        None,
                    )
                    .to_json(),
                );
                return None;
            }
        };

        // 检查IP是否在白名单中
        match self.is_ip_in_whitelist(ip_address, &ip_list) {
            Ok(true) => {
                // IP在白名单中，继续执行
                Some(input)
            }
            Ok(false) => {
                // IP不在白名单中，拒绝执行
                response.status = ChainStatus::Reject;
                response.data = Some(
                    ErrorResponse::new(
                        error_codes::FORBIDDEN,
                        format!("IP地址 {} 不在白名单中", ip_address),
                        None,
                    )
                    .to_json(),
                );
                None
            }
            Err(err) => {
                response.status = ChainStatus::Error;
                response.data = Some(
                    ErrorResponse::new(
                        error_codes::INTERNAL_ERROR,
                        format!("IP白名单检查失败: {}", err),
                        None,
                    )
                    .to_json(),
                );
                None
            }
        }
    }
}
