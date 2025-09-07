//! LinkChain - 轻量级调用链执行器
//!
//! 提供简单的链式执行功能，支持内置挂件和自定义回调

pub mod core;
pub mod chainware;
pub mod chain;
pub mod builtin;
pub mod types;
pub mod utils;

// 只公开核心API
pub use chain::ChainExecutor;
pub use chainware::config::ChainwareConfig;
pub use core::{ChainRequest, ChainResponse};
pub use types::{ErrorResponse, error_codes};

// 重新导出常用类型
pub use serde_json::Value as JsonValue;
pub use std::collections::HashMap;
