//! 链执行器模块
//!
//! 实现简化的链执行器，支持内置挂件和自定义回调

use crate::builtin::get_global_registry;
use crate::chainware::core::{Chainware, ChainwareWrapper, Closureware};
use crate::chainware::config::ChainwareConfig;
use crate::core::{ChainStatus, ChainRequest, ChainResponse};
use crate::types::{ErrorResponse, error_codes};
use std::collections::HashMap;

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

    /// 插入挂件到指定位置
    ///
    /// # 参数
    /// - `name`: 挂件名称，可以是内置挂件名称或自定义名称
    /// - `position`: 插入位置，正数从头部计算，负数从尾部倒数计算
    ///   - `None` 或 `-1`: 添加到末尾
    ///   - `0`: 插入到开头
    ///   - `-2`: 倒数第二位
    /// - `callback`: 自定义回调函数（可选）
    /// - `config`: 挂件配置（可选）
    ///
    /// # 返回
    /// - `Ok(Self)`: 成功返回链执行器
    /// - `Err(ErrorResponse)`: 失败返回错误信息
    ///
    /// # 使用方式
    /// ```ignore
    /// // 添加到末尾
    /// executor.insert_chainware("condition", None, None, Some(config));
    ///
    /// // 插入到开头
    /// executor.insert_chainware("condition", Some(0), None, Some(config));
    /// 
    /// // 插入到倒数第二位
    /// executor.insert_chainware("condition", Some(-2), None, Some(config));
    /// ```
    pub fn insert_chainware<F>(
        mut self,
        name: &str,
        position: Option<i32>,
        callback: Option<F>,
        config: Option<ChainwareConfig>,
    ) -> Result<Self, ErrorResponse>
    where
        F: Fn(
                &ChainRequest,
                &mut ChainResponse,
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
                    // 内置挂件不存在，返回错误
                    return Err(ErrorResponse::new(
                        error_codes::CONFIG_ERROR,
                        format!("未找到内置挂件: {}", name),
                        None,
                    ));
                }
            }
        };

        let wrapper = ChainwareWrapper::new(chainware, config);
        match position {
            Some(pos) => {
                let index = if pos >= 0 {
                    // 正数从头部计算
                    pos as usize
                } else {
                    // 负数从尾部倒数计算，-1表示末尾
                    let len = self.nodes.len();
                    if (-pos) as usize > len + 1 {
                        0 // 如果超出了范围，则插入到开头
                    } else {
                        len.saturating_sub((-pos) as usize - 1)
                    }
                };
                
                // 确保索引不超过当前长度
                if index <= self.nodes.len() {
                    self.nodes.insert(index, wrapper);
                } else {
                    // 如果位置超出了范围，则添加到末尾
                    self.nodes.push(wrapper);
                }
            }
            None => {
                self.nodes.push(wrapper);
            }
        }
        Ok(self)
    }
    
    /// 批量添加挂件（通过JSON数组配置）
    ///
    /// # 参数
    /// - `configs`: JSON数组，每个元素包含挂件的配置信息
    ///
    /// # JSON配置格式
    /// ```json
    /// [
    ///   {
    ///     "name": "挂件名称",
    ///     "config": { /* 挂件配置 */ }
    ///   }
    /// ]
    /// ```
    ///
    /// # 返回
    /// - `Ok(Self)`: 成功返回链执行器
    /// - `Err(ErrorResponse)`: 失败返回错误信息，包括名称、位置和错误信息
    ///
    /// # 使用方式
    /// ```ignore
    /// let configs = json!([
    ///   {
    ///     "name": "condition",
    ///     "config": {
    ///       "expression": "params.age > 18"
    ///     }
    ///   }
    /// ]);
    /// executor.add_chainwares(configs);
    /// ```
    pub fn add_chainwares(mut self, configs: serde_json::Value) -> Result<Self, ErrorResponse> {
        if let serde_json::Value::Array(chainwares) = configs {
            for (index, chainware_config) in chainwares.iter().enumerate() {
                if let serde_json::Value::Object(obj) = chainware_config {
                    let name = obj.get("name")
                        .and_then(|n| n.as_str())
                        .ok_or_else(|| ErrorResponse::new(
                            error_codes::CONFIG_ERROR,
                            format!("第{}个挂件配置缺少name字段", index + 1),
                            None,
                        ))?;
                    
                    let config = obj.get("config")
                        .map(|c| {
                            let mut config_map = HashMap::new();
                            if let serde_json::Value::Object(config_obj) = c {
                                for (key, value) in config_obj {
                                    config_map.insert(key.clone(), value.clone());
                                }
                            }
                            ChainwareConfig::new(config_map)
                        });
                    
                    self = self.insert_chainware(name, None, None::<fn(
                        &ChainRequest,
                        &mut ChainResponse,
                        Option<serde_json::Value>,
                        Option<&ChainwareConfig>,
                    ) -> Option<serde_json::Value>>, config)
                    .map_err(|e| ErrorResponse::new(
                        error_codes::CONFIG_ERROR,
                        format!("在处理第{}个挂件 '{}' 时出错: {}", index + 1, name, e.msg),
                        None,
                    ))?;
                } else {
                    return Err(ErrorResponse::new(
                        error_codes::CONFIG_ERROR,
                        format!("第{}个挂件配置必须是对象", index + 1),
                        None,
                    ));
                }
            }
        } else {
            return Err(ErrorResponse::new(
                error_codes::CONFIG_ERROR,
                "挂件配置必须是数组".to_string(),
                None,
            ));
        }
        Ok(self)
    }

    /// 执行链
    pub fn execute(&self, request: ChainRequest) -> ChainResponse {
        let mut response = ChainResponse::new(request.start_time.clone());
        // 初始化数据为请求数据
        let mut params: serde_json::Value = request.params.clone();
        response.set_data(params.clone());

        // 按顺序执行所有节点
        for (index, node) in self.nodes.iter().enumerate() {
            // 执行节点，获取返回数据
            let node_result = node.execute(&request, &mut response, Some(params.clone()));

            // 更新数据为当前节点的返回数据
            params = node_result.unwrap_or_default();

            // 根据响应状态判断是否继续执行
            match response.status {
                ChainStatus::Continue => {
                    // 继续执行下一个节点
                    response.set_data(params.clone());
                    continue;
                }
                ChainStatus::Error | ChainStatus::Reject => {
                    // 错误或拒绝，停止执行
                    // 添加详细的错误信息到响应中
                    if response.status == ChainStatus::Error {
                        let node_name = node.name();
                        response.set_meta(
                            "error_details".to_string(), 
                            serde_json::json!({
                                "node_index": index,
                                "node_name": node_name,
                                "message": format!("在执行第{}个挂件 '{}' 时发生错误", index + 1, node_name)
                            })
                        );
                    }
                    break;
                }
                ChainStatus::Completed => {
                    // 执行完成，停止执行
                    break;
                }
            }
        }

        // 如果所有节点都执行完成且状态仍为Continue，则设置为Completed
        if response.status == ChainStatus::Continue {
            response.set_status(ChainStatus::Completed);
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