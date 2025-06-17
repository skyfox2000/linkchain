//! 链执行器模块
//!
//! 实现简化的链执行器，支持内置挂件和自定义回调

use crate::builtin::get_global_registry;
use crate::chainware::core::{Chainware, ChainwareWrapper, Closureware};
use crate::chainware::config::ChainwareConfig;
use crate::core::{ExecutionStatus, RequestContext, ResponseContext};

/// 链执行器
/// 简化的链执行器，外部程序创建链后添加挂件然后执行
pub struct ChainExecutor {
    /// 挂件节点列表
    nodes: Vec<ChainwareWrapper>,
}

impl ChainExecutor {
    /// 创建新的链执行器
    pub fn new() -> Self {
        Self { nodes: Vec::new() }
    }

    /// 添加挂件（简化方法）
    ///
    /// # 参数
    /// - `name`: 挂件名称，可以是内置挂件名称或自定义名称
    /// - `callback`: 自定义回调函数（可选）
    /// - `config`: 挂件配置（可选）
    ///
    /// # 使用方式
    /// ```ignore
    /// // 使用内置挂件
    /// executor.add_chainware("condition", None, Some(config));
    ///
    /// // 使用自定义回调
    /// executor.add_chainware("my_custom", Some(callback), Some(config));
    /// ```
    pub fn add_chainware<F>(
        mut self,
        name: &str,
        callback: Option<F>,
        config: Option<ChainwareConfig>,
    ) -> Self
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
        let chainware: Box<dyn Chainware> = if let Some(cb) = callback {
            // 优先使用自定义回调
            Box::new(Closureware::new(name.to_string(), cb))
        } else {
            // 尝试使用内置挂件
            let registry = get_global_registry();
            match registry.create_chainware(name) {
                Some(chainware) => chainware,
                None => {
                    // 内置挂件不存在，直接panic
                    panic!("未找到内置挂件: {}", name);
                }
            }
        };

        let wrapper = ChainwareWrapper::new(chainware, config);
        self.nodes.push(wrapper);
        self
    }

    /// 执行链
    pub fn execute(&self, request: RequestContext) -> ResponseContext {
        let mut response = ResponseContext::new();
        // 初始化数据为请求数据
        let mut params: serde_json::Value = request.params.clone();

        // 按顺序执行所有节点
        for node in &self.nodes {
            // 执行节点，获取返回数据
            let node_result = node.execute(&request, &mut response, Some(params.clone()));

            // 更新数据为当前节点的返回数据
            params = node_result.unwrap_or_default();

            // 根据响应状态判断是否继续执行
            match response.status {
                ExecutionStatus::Continue => {
                    // 继续执行下一个节点
                    continue;
                }
                ExecutionStatus::Error | ExecutionStatus::Reject => {
                    // 错误或拒绝，停止执行
                    break;
                }
                ExecutionStatus::Completed => {
                    // 执行完成，停止执行
                    break;
                }
            }
        }

        // 如果所有节点都执行完成且状态仍为Continue，则设置为Completed
        if response.status == ExecutionStatus::Continue {
            response.set_status(ExecutionStatus::Completed);
        }

        // 设置执行结束时间
        response.set_end_time();

        response
    }
}

impl Default for ChainExecutor {
    fn default() -> Self {
        Self::new()
    }
}
