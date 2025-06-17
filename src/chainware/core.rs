//! 挂件模块
//!
//! 定义链挂件接口和相关类型

use crate::chainware::config::ChainwareConfig;
use crate::core::{ExecutionStatus, RequestContext, ResponseContext};

/// 链挂件接口（挂件接口）
/// 所有挂件都需要实现这个接口
pub trait Chainware: Send + Sync {
    /// 获取挂件名称
    fn name(&self) -> &str;

    /// 处理方法（核心方法）
    /// 挂件保持简单，主要做过滤判断和简单数据处理
    /// 参数：请求上下文、响应上下文、外部数据及上一个挂件返回数据
    /// 参数：挂件配置信息
    /// 返回：挂件的返回数据
    fn process(
        &self,
        request: &RequestContext,
        response: &mut ResponseContext,
        data: Option<serde_json::Value>,
        config: Option<&ChainwareConfig>,
    ) -> Option<serde_json::Value>;
}

/// 挂件包装器
/// 用于包装实际的挂件实现，提供配置支持
pub struct ChainwareWrapper {
    /// 挂件实现
    node: Box<dyn Chainware>,
    /// 挂件配置
    config: Option<ChainwareConfig>,
}

impl ChainwareWrapper {
    /// 创建新的挂件包装器
    pub fn new(node: Box<dyn Chainware>, config: Option<ChainwareConfig>) -> Self {
        Self { node, config }
    }

    /// 检查挂件是否启用
    fn is_enabled(&self) -> bool {
        // 优先检查配置中的启用状态
        self.config
            .as_ref()
            .is_none_or(|config| config.get_enabled())
    }

    /// 执行挂件处理
    pub fn execute(
        &self,
        request: &RequestContext,
        response: &mut ResponseContext,
        data: Option<serde_json::Value>,
    ) -> Option<serde_json::Value> {
        if self.is_enabled() {
            let result = self
                .node
                .process(request, response, data, self.config.as_ref());
            if response.status == ExecutionStatus::Continue {
                response.data = result.clone();
            }
            result
        } else {
            None
        }
    }
}

/// 处理器函数类型定义
type ProcessorFn = Box<
    dyn Fn(
            &RequestContext,
            &mut ResponseContext,
            Option<serde_json::Value>,
            Option<&ChainwareConfig>,
        ) -> Option<serde_json::Value>
        + Send
        + Sync,
>;

/// 闭包挂件实现
/// 提供一个基础的挂件实现，可以通过闭包自定义处理逻辑
pub struct Closureware {
    /// 挂件名称
    name: String,
    /// 处理函数
    /// 参数：请求上下文、响应上下文、外部数据或上一个挂件返回数据
    /// 参数：挂件配置信息
    /// 返回：挂件的返回数据
    processor: ProcessorFn,
}

impl Closureware {
    /// 创建新的闭包挂件
    pub fn new<F>(name: String, processor: F) -> Self
    where
        F: Fn(
                &RequestContext,
                &mut ResponseContext,
                Option<serde_json::Value>,
                Option<&ChainwareConfig>,
            ) -> Option<serde_json::Value>
            + Send
            + Sync
            + 'static,
    {
        Self {
            name,
            processor: Box::new(processor),
        }
    }
}

impl Chainware for Closureware {
    fn name(&self) -> &str {
        &self.name
    }

    fn process(
        &self,
        request: &RequestContext,
        response: &mut ResponseContext,
        data: Option<serde_json::Value>,
        config: Option<&ChainwareConfig>,
    ) -> Option<serde_json::Value> {
        (self.processor)(request, response, data, config)
    }
}
