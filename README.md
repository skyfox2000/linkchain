# LinkChain - 轻量级链执行器

LinkChain 是一个轻量级的链执行器，专为简单、高效的数据处理管道而设计。通过配置驱动的挂件系统，实现数据在挂件之间的流式处理。

## 核心特性

- **流水线处理**：数据在挂件之间按顺序流动，每个挂件处理并传递数据
- **配置驱动**：通过JSON配置控制挂件行为，无需编写代码
- **状态控制**：支持Continue、Completed、Error、Reject等执行状态
- **上下文传递**：支持`$.`、`$params.`、`$meta.`等上下文数据访问
- **JSON输出**：所有结果以JSON格式输出

### JSONPath 路径规范和执行状态

**执行状态类型**：
- **Continue**: 继续执行下一个挂件
- **Completed**: 链执行完成，返回最终结果  
- **Error**: 数据处理异常错误，中断执行
- **Reject**: 条件不满足被拒绝，中断执行

**JSONPath 路径规范**：

**基础语法**：
- `$.field` 或 `$input.field` - 访问当前输入数据字段（上一个挂件的输出）
- `$params.key` - 访问原始请求参数（整个链中不变）
- `$meta.info` - 访问元数据信息（如IP地址、时间戳等）

**数组访问示例**：
```
$.array[0]          # 访问数组第一个元素
$.items[1].name     # 访问数组第二个元素的name字段
$.users[-1]         # 访问数组最后一个元素
$.products[0:3]     # 访问数组前3个元素（切片）
```

**嵌套对象访问示例**：
```
$.user.profile.name           # 深层嵌套字段
$.data.users[0].contact.email # 数组元素的嵌套字段
$.config.settings.theme       # 多层配置访问
```

**长度和数量获取**：
```
$.items.length        # 数组长度
$.username.length     # 字符串长度  
$.settings.length     # 对象属性数量
$.nested.array.length # 嵌套数组长度
```

**跨上下文数据访问示例**：
```
$params.user_id              # 原始请求参数（不变）
$params.config.theme         # 原始配置参数（不变）
$meta.ip_address             # 客户端IP地址（不变）
$meta.timestamp              # 请求时间戳（不变）
$meta.session_id             # 会话ID（不变）
$.current_data               # 当前挂件输入数据
```

**模板字符串示例**：
```
"用户 ${$.name} 来自 ${$.city}"                    # 简单模板
"请求来自 ${$meta.ip_address} 在 ${$meta.timestamp}" # 元数据模板
"配置: ${$params.theme}, 当前值: ${$.value}"        # 混合数据源
```

**条件表达式中的路径**：
```
$.age >= 18                           # 数值比较
$.role == "admin"                     # 字符串比较
$.items.length > 0                    # 长度检查
$params.debug_mode == true            # 参数检查
String.endsWith($.email, "@company.com") # 字符串函数
```

## 快速开始

### 基本用法

```rust
use linkchain::{ChainExecutor, RequestContext, ChainwareConfig};
use serde_json::json;
use std::collections::HashMap;

// 创建链执行器
let mut executor = ChainExecutor::new();

// 添加条件判断挂件
let mut condition_config = HashMap::new();
condition_config.insert("expression".to_string(), json!("$.age >= 18"));
executor = executor.add_chainware("condition", None, Some(ChainwareConfig { config: condition_config }));

// 添加日志记录挂件
let mut logger_config = HashMap::new();
logger_config.insert("template".to_string(), json!("用户 ${$.name} 已验证，年龄: ${$.age}"));
executor = executor.add_chainware("logger", None, Some(ChainwareConfig { config: logger_config }));

// 创建请求上下文
let input_data = json!({"name": "张三", "age": 25, "city": "北京"});
let context = RequestContext::new(input_data);

// 执行链
let response = executor.execute(context);

// 检查执行结果
match response.status {
    ExecutionStatus::Completed => println!("执行成功: {:?}", response.data),
    ExecutionStatus::Error => println!("执行错误: {:?}", response.data),
    ExecutionStatus::Reject => println!("执行被拒绝"),
    _ => println!("其他状态: {:?}", response.status),
}
```

## 内置挂件详细配置

### 1. condition - 条件判断挂件

**功能**：基于表达式进行条件判断，支持多种比较和逻辑操作

**配置参数**：
- `expression` 或 `condition` (String): 条件表达式

**基础比较示例**：
```rust
// 数值比较
let mut config = HashMap::new();
config.insert("expression".to_string(), json!("$.age >= 18"));

// 字符串相等比较
config.insert("expression".to_string(), json!("$.role == \"admin\""));

// 不等比较
config.insert("expression".to_string(), json!("$.status != \"inactive\""));
```

**字符串操作示例**：
```rust
// 字符串前缀检查
config.insert("expression".to_string(), json!("String.startsWith($.email, \"admin@\")"));

// 字符串后缀检查
config.insert("expression".to_string(), json!("String.endsWith($.email, \"@company.com\")"));

// 字符串包含检查
config.insert("expression".to_string(), json!("String.contains($.message, \"error\")"));

// 正则匹配
config.insert("expression".to_string(), json!("String.matches($.phone, \"^1[3-9]\\\\d{9}$\")"));
```

**类型检查示例**：
```rust
// 检查是否为字符串
config.insert("expression".to_string(), json!("Chain.isString($.name)"));

// 检查是否为数字
config.insert("expression".to_string(), json!("Chain.isNumber($.age)"));

// 检查是否为空
config.insert("expression".to_string(), json!("Chain.isEmpty($.description)"));

// 检查是否为数组
config.insert("expression".to_string(), json!("Chain.isArray($.items)"));
```

**长度检查示例**：
```rust
// 字符串长度检查
config.insert("expression".to_string(), json!("$.username.length >= 6"));

// 数组长度检查
config.insert("expression".to_string(), json!("$.items.length > 0"));

// 对象属性数量检查
config.insert("expression".to_string(), json!("$.settings.length <= 10"));
```

**复杂逻辑组合示例**：
```rust
// AND条件组合
config.insert("expression".to_string(), json!(
    "$.age >= 18 && $.role == \"user\" && String.endsWith($.email, \"@company.com\")"
));

// OR条件组合
config.insert("expression".to_string(), json!(
    "$.role == \"admin\" || $.role == \"moderator\" || $.permissions.length > 5"
));

// 混合逻辑
config.insert("expression".to_string(), json!(
    "($.age >= 18 && $.verified == true) || $.role == \"admin\""
));
```

---

### 2. logger - 日志记录挂件

**功能**：记录执行过程中的信息，支持模板变量替换

**配置参数**：
- `template` (String): 日志模板，支持 `${variable}` 变量替换

**简单日志示例**：
```rust
let mut config = HashMap::new();
config.insert("template".to_string(), json!("用户 ${$.username} 登录成功"));
```

**复杂模板示例**：
```rust
// 多字段日志
config.insert("template".to_string(), json!(
    "用户信息: 姓名=${$.name}, 年龄=${$.age}, 邮箱=${$.email}, 角色=${$.role}"
));

// 包含元数据的日志
config.insert("template".to_string(), json!(
    "请求详情: IP=${$meta.ip_address}, 用户=${$.username}, 时间=${$meta.timestamp}, 操作=${$.action}"
));

// 嵌套对象访问
config.insert("template".to_string(), json!(
    "用户 ${$.user.profile.name} 从 ${$.user.location.city} 发起请求"
));

// 数组信息日志
config.insert("template".to_string(), json!(
    "订单处理: 用户=${$.customer.name}, 商品数量=${$.items.length}, 总金额=${$.total}"
));
```

---

### 3. merge - 数据合并挂件

**功能**：将其他路径的数据合并到当前数据中

**配置参数**：
- `data_path` (String): 要合并的数据路径

**简单合并示例**：
```rust
// 合并参数数据
let mut config = HashMap::new();
config.insert("data_path".to_string(), json!("$params.extra_info"));
```

**复杂合并示例**：
```rust
// 合并嵌套对象
config.insert("data_path".to_string(), json!("$params.user.preferences"));

// 合并元数据
config.insert("data_path".to_string(), json!("$meta.session"));

// 多级路径合并
config.insert("data_path".to_string(), json!("$params.config.default_settings"));
```

---

### 4. map_fields - 字段映射挂件

**功能**：对象字段重命名和转换，支持两种模式

**配置参数**：
- `mapping` (Object): 字段映射配置，key为新字段名，value为源路径
- `overwrite` (Boolean): 是否覆盖模式，默认true

**基础映射示例**：
```rust
let mut config = HashMap::new();
config.insert("mapping".to_string(), json!({
    "username": "$.name",
    "user_age": "$.age",
    "location": "$.city"
}));
config.insert("overwrite".to_string(), json!(true)); // 保留原字段
```

**严格模式示例**：
```rust
// 只保留映射的字段
config.insert("mapping".to_string(), json!({
    "id": "$.user_id",
    "email": "$.email_address",
    "role": "$.permissions.role"
}));
config.insert("overwrite".to_string(), json!(false)); // 只输出映射字段
```

**复杂映射示例**：
```rust
// 跨层级字段映射
config.insert("mapping".to_string(), json!({
    "full_name": "$.user.profile.first_name",
    "contact_email": "$.user.contact.email",
    "primary_role": "$.user.permissions.roles[0]",
    "last_login": "$meta.last_access_time",
    "setting_theme": "$params.user_preferences.theme"
}));

// 数组元素映射
config.insert("mapping".to_string(), json!({
    "first_item_name": "$.items[0].name",
    "first_item_price": "$.items[0].price",
    "total_items": "$.items.length"
}));
```

**数组对象批量映射示例**：
```rust
// 当输入是数组时，会对每个元素进行映射
config.insert("mapping".to_string(), json!({
    "product_name": "$.name",
    "product_price": "$.price",
    "discounted_price": "$.sale_price",
    "category": "$.category.name"
}));
// 输入: [{"name": "商品A", "price": 100, "category": {"name": "电子"}}]
// 输出: [{"name": "商品A", "price": 100, "category": {"name": "电子"}, 
//         "product_name": "商品A", "product_price": 100, "category": "电子"}]
```

---

### 5. extract_map - 提取映射挂件

**功能**：从输入数据中提取指定字段，组成新对象返回

**配置参数**：
- `mapping` (Object): 提取映射配置，key为新字段名，value为源路径或模板

**基础提取示例**：
```rust
let mut config = HashMap::new();
config.insert("mapping".to_string(), json!({
    "user_name": "$.name",
    "user_age": "$.age",
    "location": "$.city"
}));
```

**复杂提取示例**：
```rust
// 多数据源提取
config.insert("mapping".to_string(), json!({
    "username": "$.user.profile.name",
    "email": "$.user.contact.email", 
    "full_address": "${$.address.street} ${$.address.city}",
    "role": "default_user",  // 字面量
    "timestamp": "$meta.current_time",
    "first_item": "$.items[0].name",  // 数组访问
    "param_setting": "$params.config.theme"
}));
```

**模板字符串示例**：
```rust
// 组合字段提取
config.insert("mapping".to_string(), json!({
    "display_name": "${$.first_name} ${$.last_name}",
    "contact_info": "邮箱: ${$.email}, 电话: ${$.phone}",
    "summary": "用户 ${$.name} 来自 ${$.city}, 年龄 ${$.age} 岁"
}));
```

---

### 6. extract_json - 结构化JSON提取挂件

**功能**：从文本中提取JSON对象并进行结构化处理

**配置参数**：
- 无需配置参数

**示例**：
```rust
// 从包含JSON的文本中提取JSON对象
let input = json!("用户信息：{\"name\":\"张三\",\"age\":25} 处理完成");
// 输出：{"name":"张三","age":25}

// 从纯JSON字符串中提取
let input = json!("{\"status\":\"success\",\"data\":[1,2,3]}");
// 输出：{"status":"success","data":[1,2,3]}
```

---

### 7. extract_sql - SQL语句提取挂件

**功能**：从文本中提取SQL语句，支持Markdown代码块和直接SQL语句

**配置参数**：
- 无需配置参数

**Markdown代码块示例**：
```rust
// 提取Markdown SQL代码块
let input = json!("查询语句如下：\n```sql\nSELECT * FROM users WHERE age > 18;\n```\n执行结果...");
// 输出："SELECT * FROM users WHERE age > 18;"
```

**直接SQL语句示例**：
```rust
// 提取直接的SQL语句
let input = json!("需要执行 INSERT INTO users (name, age) VALUES ('张三', 25); 来添加用户");
// 输出："INSERT INTO users (name, age) VALUES ('张三', 25);"

// 复杂SQL语句
let input = json!("UPDATE products SET price = price * 0.9 WHERE category = 'electronics';");
// 输出："UPDATE products SET price = price * 0.9 WHERE category = 'electronics';"
```

**支持的SQL类型**：
- SELECT查询语句
- INSERT插入语句  
- UPDATE更新语句
- DELETE删除语句
- CREATE创建语句
- ALTER修改语句
- DROP删除语句
- 事务控制语句（COMMIT、ROLLBACK等）

---

### 8. json_extract - JSON数据提取挂件

**功能**：使用JSONPath从数据中提取特定值

**配置参数**：
- `pattern` (String): JSONPath路径表达式

**简单提取示例**：
```rust
let mut config = HashMap::new();
config.insert("pattern".to_string(), json!("$.user.name"));
```

**复杂提取示例**：
```rust
// 数组过滤提取
config.insert("pattern".to_string(), json!("$.orders[?(@.status == 'completed')].amount"));

// 多层级提取
config.insert("pattern".to_string(), json!("$.data.users[*].profile.email"));

// 条件提取
config.insert("pattern".to_string(), json!("$.products[?(@.price > 100)].name"));
```

---

### 9. regexp_extract - 正则表达式提取挂件

**功能**：使用正则表达式从文本中提取数据

**配置参数**：
- `pattern` (String): 正则表达式模式

**简单提取示例**：
```rust
let mut config = HashMap::new();
config.insert("pattern".to_string(), json!(r"(\d{4}-\d{2}-\d{2})"));
```

**复杂提取示例**：
```rust
// 提取邮箱
config.insert("pattern".to_string(), json!(r"([a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,})"));

// 提取手机号
config.insert("pattern".to_string(), json!(r"(1[3-9]\d{9})"));

// 提取多个分组
config.insert("pattern".to_string(), json!(r"用户(\w+)在(\d{4}-\d{2}-\d{2})执行了(.+)操作"));

// 提取URL参数
config.insert("pattern".to_string(), json!(r"https?://[^/]+/([^?]+)\?(.+)"));
```

---

### 10. regexp_condition - 正则表达式条件挂件

**功能**：使用正则表达式进行条件判断

**配置参数**：
- `pattern` (String): 正则表达式模式

**格式验证示例**：
```rust
// 用户名格式验证
let mut config = HashMap::new();
config.insert("pattern".to_string(), json!(r"^[a-zA-Z0-9_]{3,20}$"));

// 邮箱格式验证
config.insert("pattern".to_string(), json!(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$"));

// 手机号验证
config.insert("pattern".to_string(), json!(r"^1[3-9]\d{9}$"));

// 身份证号验证
config.insert("pattern".to_string(), json!(r"^[1-9]\d{5}(18|19|20)\d{2}(0[1-9]|1[0-2])(0[1-9]|[12]\d|3[01])\d{3}[\dXx]$"));
```

**内容检查示例**：
```rust
// 检查是否包含敏感词
config.insert("pattern".to_string(), json!(r"(广告|spam|垃圾)"));

// 检查URL格式
config.insert("pattern".to_string(), json!(r"^https?://"));

// 检查IP地址格式
config.insert("pattern".to_string(), json!(r"^(\d{1,3}\.){3}\d{1,3}$"));
```

---

### 11. ip_blacklist - IP黑名单挂件

**功能**：检查IP地址是否在黑名单中

**配置参数**：
- `ip_list` (String): 黑名单IP列表，逗号分隔

**单IP黑名单示例**：
```rust
let mut config = HashMap::new();
config.insert("ip_list".to_string(), json!("192.168.1.100,10.0.0.50,172.16.0.100"));
```

**CIDR网段黑名单示例**：
```rust
// 支持网段格式
config.insert("ip_list".to_string(), json!("192.168.1.0/24,10.0.0.0/8,172.16.0.0/16"));

// 混合格式
config.insert("ip_list".to_string(), json!("192.168.1.100,10.0.0.0/8,172.16.0.50,192.168.0.0/16"));
```

**实际应用示例**：
```rust
// 阻止特定攻击IP
config.insert("ip_list".to_string(), json!("1.2.3.4,5.6.7.8,malicious.attacker.com"));

// 阻止整个地区网段
config.insert("ip_list".to_string(), json!("185.220.0.0/16,198.98.0.0/16"));
```

---

### 12. ip_whitelist - IP白名单挂件

**功能**：检查IP地址是否在白名单中，只允许白名单IP通过

**配置参数**：
- `ip_list` (String): 白名单IP列表，逗号分隔

**内网白名单示例**：
```rust
let mut config = HashMap::new();
config.insert("ip_list".to_string(), json!("127.0.0.1,192.168.0.0/16,10.0.0.0/8,172.16.0.0/12"));
```

**特定办公室IP示例**：
```rust
// 只允许特定办公室访问
config.insert("ip_list".to_string(), json!("203.0.113.10,203.0.113.11,203.0.113.0/24"));

// 允许CDN和特定用户
config.insert("ip_list".to_string(), json!("8.8.8.8,8.8.4.4,1.1.1.1,192.168.1.100"));
```

**开发环境示例**：
```rust
// 开发测试环境白名单
config.insert("ip_list".to_string(), json!("127.0.0.1,::1,192.168.1.0/24,10.0.0.0/24"));
```

## 完整应用示例

### 用户认证和权限检查链

```rust
use linkchain::{ChainExecutor, RequestContext, ChainwareConfig};
use serde_json::json;
use std::collections::HashMap;

fn create_auth_chain() -> ChainExecutor {
    let mut executor = ChainExecutor::new();

    // 1. IP白名单检查
    let mut ip_config = HashMap::new();
    ip_config.insert("ip_list".to_string(), json!("192.168.0.0/16,10.0.0.0/8,127.0.0.1"));
    executor = executor.add_chainware("ip_whitelist", None, 
        Some(ChainwareConfig { config: ip_config }));

    // 2. 用户名格式验证
    let mut username_config = HashMap::new();
    username_config.insert("pattern".to_string(), json!(r"^[a-zA-Z0-9_]{3,20}$"));
    executor = executor.add_chainware("regexp_condition", None, 
        Some(ChainwareConfig { config: username_config }));

    // 3. 用户角色权限检查
    let mut role_config = HashMap::new();
    role_config.insert("expression".to_string(), 
        json!("$.role == \"admin\" || $.role == \"moderator\" || ($.role == \"user\" && $.verified == true)"));
    executor = executor.add_chainware("condition", None, 
        Some(ChainwareConfig { config: role_config }));

    // 4. 记录认证日志
    let mut log_config = HashMap::new();
    log_config.insert("template".to_string(), 
        json!("用户认证成功: 用户=${$.username}, 角色=${$.role}, IP=${$meta.ip_address}, 时间=${$meta.timestamp}"));
    executor = executor.add_chainware("logger", None, 
        Some(ChainwareConfig { config: log_config }));

    // 5. 提取用户信息
    let mut extract_config = HashMap::new();
    extract_config.insert("mapping".to_string(), json!({
        "user_id": "$.id",
        "username": "$.username",
        "role": "$.role",
        "permissions": "$.permissions",
        "login_time": "$meta.timestamp",
        "client_ip": "$meta.ip_address",
        "session_id": "$meta.session_id"
    }));
    executor = executor.add_chainware("extract_map", None, 
        Some(ChainwareConfig { config: extract_config }));

    executor
}

fn main() {
    let executor = create_auth_chain();
    
    // 创建请求上下文
    let input_data = json!({
        "username": "admin_user",
        "id": 1001,
        "role": "admin",
        "verified": true,
        "permissions": ["read", "write", "delete"]
    });
    
    let mut context = RequestContext::new(input_data);
    context.meta.insert("ip_address".to_string(), json!("192.168.1.100"));
    context.meta.insert("timestamp".to_string(), json!("2024-01-01T10:00:00Z"));
    context.meta.insert("session_id".to_string(), json!("session_12345"));
    
    // 执行认证链
    let response = executor.execute(context);
    
    match response.status {
        ExecutionStatus::Completed => {
            println!("认证成功: {:?}", response.data);
        }
        ExecutionStatus::Reject => {
            println!("认证被拒绝");
        }
        ExecutionStatus::Error => {
            println!("认证过程出错: {:?}", response.data);
        }
        _ => {
            println!("未知状态: {:?}", response.status);
        }
    }
}
```

### 数据处理和验证链

```rust
fn create_data_processing_chain() -> ChainExecutor {
    let mut executor = ChainExecutor::new();

    // 1. 数据格式验证
    let mut format_config = HashMap::new();
    format_config.insert("expression".to_string(), 
        json!("Chain.isObject($.data) && Chain.isString($.data.email) && Chain.isNumber($.data.age)"));
    executor = executor.add_chainware("condition", None, 
        Some(ChainwareConfig { config: format_config }));

    // 2. 邮箱格式验证
    let mut email_config = HashMap::new();
    email_config.insert("pattern".to_string(), 
        json!(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$"));
    executor = executor.add_chainware("regexp_condition", None, 
        Some(ChainwareConfig { config: email_config }));

    // 3. 年龄范围检查
    let mut age_config = HashMap::new();
    age_config.insert("expression".to_string(), json!("$.data.age >= 18 && $.data.age <= 120"));
    executor = executor.add_chainware("condition", None, 
        Some(ChainwareConfig { config: age_config }));

    // 4. 合并默认配置
    let mut merge_config = HashMap::new();
    merge_config.insert("data_path".to_string(), json!("$params.default_settings"));
    executor = executor.add_chainware("merge", None, 
        Some(ChainwareConfig { config: merge_config }));

    // 5. 字段映射标准化
    let mut map_config = HashMap::new();
    map_config.insert("mapping".to_string(), json!({
        "user_email": "$.data.email",
        "user_age": "$.data.age",
        "user_name": "$.data.name",
        "created_at": "$meta.timestamp",
        "source": "$params.source"
    }));
    map_config.insert("overwrite".to_string(), json!(false)); // 只保留映射字段
    executor = executor.add_chainware("map_fields", None, 
        Some(ChainwareConfig { config: map_config }));

    executor
}
```

## 重要设计原则

### 数据不变性保证

**$params 和 $meta 数据不变性**：
- `$params` - 原始请求参数在整个链执行过程中**严格不变**
- `$meta` - 元数据信息在整个链执行过程中**严格不变**  
- `$.` - 当前输入数据会在挂件间流转和变化

这种设计确保了：
1. **追溯性**: 任何时候都可以访问原始请求数据
2. **一致性**: 元数据如IP地址、时间戳等保持不变  
3. **安全性**: 避免意外修改原始请求参数

### 配置覆盖完整性

本文档覆盖了**所有11个内置挂件**的完整配置说明：

✅ **条件和验证类**：
- `condition` - 条件判断挂件（基础比较、逻辑组合、字符串/类型检查）
- `regexp_condition` - 正则条件挂件（格式验证、内容检查）

✅ **数据提取类**：  
- `extract_json` - 从文本提取JSON对象/数组
- `extract_sql` - 从文本提取SQL语句
- `json_extract` - JSONPath数据提取
- `regexp_extract` - 正则表达式数据提取

✅ **数据处理类**：
- `extract_map` - 提取映射（创建新对象）
- `map_fields` - 字段映射（修改现有对象）
- `merge` - 数据合并
- `logger` - 日志记录

✅ **网络安全类**：
- `ip_blacklist` - IP黑名单过滤
- `ip_whitelist` - IP白名单过滤

每个挂件都包含：
- 功能说明
- 配置参数详解  
- 基础使用示例
- 复杂应用场景
- 实际项目示例

## 注意事项

1. **挂件执行顺序**：挂件按添加顺序依次执行，顺序很重要
2. **数据流转**：每个挂件的输出作为下一个挂件的输入
3. **状态控制**：遇到Error或Reject状态时链会终止执行
4. **路径安全**：使用JSONPath时注意路径的存在性，不存在的路径返回null
5. **配置验证**：确保配置参数类型和格式正确，错误的配置会导致挂件执行失败
6. **元数据传递**：`$meta.ip_address`等元数据需要在创建RequestContext时设置
7. **数据不变性**：`$params`和`$meta`在整个执行链中保持不变，只有`$.`会在挂件间流转

## 许可证

MIT License
