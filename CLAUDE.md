# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## 项目概览

LinkChain 是一个轻量级的链执行器，专为简单、高效的数据处理管道而设计。通过配置驱动的挂件系统，实现数据在挂件之间的流式处理。

## 核心架构

### 主要组件
- **ChainExecutor**: 链执行器核心，管理挂件执行顺序
- **Chainware**: 挂件接口，所有内置和自定义挂件都实现此接口
- **ChainRequest/ChainResponse**: 请求/响应上下文，包含执行状态和数据
- **Builtin Chainwares**: 12个内置挂件，涵盖条件判断、数据处理、IP过滤等功能

### 执行流程
1. 创建 ChainExecutor 实例
2. 按顺序添加挂件（内置或自定义）
3. 创建 ChainRequest 包含输入数据和元数据
4. 执行链，数据在挂件间流转
5. 返回 ChainResponse 包含最终结果和执行状态

### 数据流模型
- `$params`: 原始请求参数（整个链中不变）
- `$meta`: 元数据信息（整个链中不变）
- `$.`: 当前输入数据（在挂件间流转变化）

## 开发命令

### 构建项目
```bash
cargo build          # 开发构建
cargo build --release # 发布构建
```

### 运行测试
```bash
cargo test           # 运行所有测试
cargo test --release # 发布模式运行测试

# 运行特定测试文件
cargo test test_condition_chainwares::tests
cargo test test_ip_filter_chainwares::tests
cargo test test_extract_chainwares::tests
cargo test test_data_processing_chainwares::tests
cargo test test_integration_scenarios::tests

# 运行单个测试用例
cargo test test_name::test_function
```

### 代码质量检查
```bash
cargo clippy        # 代码风格检查
cargo fmt --check   # 代码格式化检查
```

### 完整测试套件
```bash
./scripts/run_comprehensive_tests.sh  # 运行完整的统一测试脚本
```

## 内置挂件类型

### 条件和验证类
- `condition`: 条件判断挂件（基础比较、逻辑组合）
- `regexp_condition`: 正则条件挂件（格式验证、内容检查）

### 数据提取类
- `extract_json`: 从文本提取JSON对象/数组
- `extract_sql`: 从文本提取SQL语句
- `json_extract`: JSONPath数据提取
- `regexp_extract`: 正则表达式数据提取

### 数据处理类
- `extract_map`: 提取映射（创建新对象）
- `map_fields`: 字段映射（修改现有对象）
- `merge`: 数据合并
- `logger`: 日志记录

### 网络安全类
- `ip_blacklist`: IP黑名单过滤
- `ip_whitelist`: IP白名单过滤

## JSONPath 路径规范

### 基础语法
- `$.field` - 访问当前输入数据字段
- `$params.key` - 访问原始请求参数
- `$meta.info` - 访问元数据信息

### 模板字符串
```
"用户 ${$.name} 来自 ${$.city}"
"请求来自 ${$meta.ip_address} 在 ${$meta.timestamp}"
```

## 执行状态

- **Continue**: 继续执行下一个挂件
- **Completed**: 链执行完成，返回最终结果
- **Error**: 数据处理异常错误，中断执行
- **Reject**: 条件不满足被拒绝，中断执行

## 开发注意事项

1. 挂件执行顺序很重要，按添加顺序依次执行
2. 数据在挂件间流转，每个挂件的输出作为下一个挂件的输入
3. `$params` 和 `$meta` 在整个执行链中保持不变
4. 使用 JSONPath 时注意路径的存在性，不存在的路径返回 null
5. 配置验证确保参数类型和格式正确