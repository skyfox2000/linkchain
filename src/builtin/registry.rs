//! 内置挂件注册表
//!
//! 管理所有内置挂件的注册和查找

use crate::chainware::core::{Chainware};
use super::{
    ConditionChainware, ExtractJsonChainware, ExtractMapChainware, ExtractSqlChainware, JsonExtractChainware, LoggerChainware,
    MapFieldsChainware, MergeChainware, RegexpConditionChainware, RegexpExtractChainware,
    IpBlacklistChainware, IpWhitelistChainware,
};
use std::collections::HashMap;

use std::sync::OnceLock;

/// 全局内置挂件注册表实例
static GLOBAL_REGISTRY: OnceLock<BuiltinChainwareRegistry> = OnceLock::new();

/// 获取全局注册表实例
pub fn get_global_registry() -> &'static BuiltinChainwareRegistry {
    GLOBAL_REGISTRY.get_or_init(BuiltinChainwareRegistry::new)
}

/// 内置挂件类型枚举
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum BuiltinChainwareType {
    Condition,
    Logger,
    Merge,
    JsonExtract,
    MapFields,
    ExtractMap,
    ExtractJson,
    ExtractSql,
    RegexpExtract,
    RegexpCondition,
    IpBlacklist,
    IpWhitelist,
    Unknown(String),
}

impl From<&str> for BuiltinChainwareType {
    fn from(name: &str) -> Self {
        match name.to_lowercase().as_str() {
            "condition" => BuiltinChainwareType::Condition,
            "logger" => BuiltinChainwareType::Logger,
            "merge" => BuiltinChainwareType::Merge,
            "json_extract" => BuiltinChainwareType::JsonExtract,
            "map_fields" => BuiltinChainwareType::MapFields,
            "extract_map" => BuiltinChainwareType::ExtractMap,
            "extract_json" => BuiltinChainwareType::ExtractJson,
            "extract_sql" => BuiltinChainwareType::ExtractSql,
            "regexp_extract" => BuiltinChainwareType::RegexpExtract,
            "regexp_condition" => BuiltinChainwareType::RegexpCondition,
            "ip_blacklist" => BuiltinChainwareType::IpBlacklist,
            "ip_whitelist" => BuiltinChainwareType::IpWhitelist,
            other => BuiltinChainwareType::Unknown(other.to_string()),
        }
    }
}

/// 内置挂件注册表
pub struct BuiltinChainwareRegistry {
    chainware_types: HashMap<String, BuiltinChainwareType>,
}

impl BuiltinChainwareRegistry {
    /// 创建新的注册表
    pub fn new() -> Self {
        let mut registry = Self {
            chainware_types: HashMap::new(),
        };

        // 注册内置挂件
        registry.register("condition", BuiltinChainwareType::Condition);
        registry.register("extract_json", BuiltinChainwareType::ExtractJson);
        registry.register("extract_map", BuiltinChainwareType::ExtractMap);
        registry.register("extract_sql", BuiltinChainwareType::ExtractSql);
        registry.register("json_extract", BuiltinChainwareType::JsonExtract);
        registry.register("logger", BuiltinChainwareType::Logger);
        registry.register("map_fields", BuiltinChainwareType::MapFields);
        registry.register("merge", BuiltinChainwareType::Merge);
        registry.register("regexp_condition", BuiltinChainwareType::RegexpCondition);
        registry.register("regexp_extract", BuiltinChainwareType::RegexpExtract);
        registry.register("ip_blacklist", BuiltinChainwareType::IpBlacklist);
        registry.register("ip_whitelist", BuiltinChainwareType::IpWhitelist);

        registry
    }

    /// 注册挂件类型
    pub fn register(&mut self, name: &str, chainware_type: BuiltinChainwareType) {
        self.chainware_types
            .insert(name.to_lowercase(), chainware_type);
    }

    /// 获取挂件类型
    pub fn get_type(&self, name: &str) -> Option<&BuiltinChainwareType> {
        self.chainware_types.get(&name.to_lowercase())
    }

    /// 创建内置挂件实例
    pub fn create_chainware(&self, name: &str) -> Option<Box<dyn Chainware>> {
        match self.get_type(name)? {
            BuiltinChainwareType::Condition => Some(Box::new(ConditionChainware::new())),
            BuiltinChainwareType::Logger => Some(Box::new(LoggerChainware::new())),
            BuiltinChainwareType::Merge => Some(Box::new(MergeChainware::new())),
            BuiltinChainwareType::JsonExtract => Some(Box::new(JsonExtractChainware::new())),
            BuiltinChainwareType::MapFields => Some(Box::new(MapFieldsChainware::new())),
            BuiltinChainwareType::ExtractMap => Some(Box::new(ExtractMapChainware::new())),
            BuiltinChainwareType::ExtractJson => Some(Box::new(ExtractJsonChainware::new())),
            BuiltinChainwareType::ExtractSql => Some(Box::new(ExtractSqlChainware::new())),
            BuiltinChainwareType::RegexpExtract => Some(Box::new(RegexpExtractChainware::new())),
            BuiltinChainwareType::RegexpCondition => {
                Some(Box::new(RegexpConditionChainware::new()))
            }
            BuiltinChainwareType::IpBlacklist => Some(Box::new(IpBlacklistChainware::new())),
            BuiltinChainwareType::IpWhitelist => Some(Box::new(IpWhitelistChainware::new())),
            BuiltinChainwareType::Unknown(_) => None,
        }
    }
}

impl Default for BuiltinChainwareRegistry {
    fn default() -> Self {
        Self::new()
    }
}
