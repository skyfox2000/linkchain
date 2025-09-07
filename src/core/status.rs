//! 执行状态模块
//!
//! 定义链执行过程中的各种状态

/// 执行状态枚举
/// 核心状态为 Continue，执行器根据状态判断是否继续执行
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChainStatus {
    /// 继续执行（核心状态）
    Continue,
    /// 执行完成
    Completed,
    /// 数据异常错误
    Error,
    /// 拒绝执行
    Reject,
}