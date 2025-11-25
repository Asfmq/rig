# Rig Agent 快速参考指南

快速查找常用的 Agent 模式和代码片段。

## 📋 目录

- [基础设置](#基础设置)
- [创建 Agent](#创建-agent)
- [工具定义](#工具定义)
- [编排模式](#编排模式)
- [Pipeline 操作](#pipeline-操作)
- [常用配置](#常用配置)

---

## 基础设置

### 导入依赖

```rust
use rig::prelude::*;
use rig::completion::{Prompt, ToolDefinition};
use rig::tool::Tool;
use rig::agent::{Agent, AgentBuilder};
use serde::{Deserialize, Serialize};
use serde_json::json;
```

### Cargo.toml

```toml
[dependencies]
rig-core = "0.x"
tokio = { version = "1", features = ["full"] }
anyhow = "1.0"
serde = { version = "1", features = ["derive"] }
serde_json = "1.0"
```

### 环境变量

```bash
export OPENAI_API_KEY="your-key-here"
export ANTHROPIC_API_KEY="your-key-here"
```

---

## 创建 Agent

### 最简单的 Agent

```rust
let agent = client
    .agent("gpt-4o")
    .build();
```

### 带 Preamble

```rust
let agent = client
    .agent("gpt-4o")
    .preamble("你是专业助手")
    .build();
```

### 带上下文

```rust
let agent = client
    .agent("gpt-4o")
    .preamble("你是客服")
    .context("公司：ABC有限公司")
    .context("营业时间：9-18点")
    .build();
```

### 完整配置

```rust
let agent = client
    .agent("gpt-4o")
    .name("助手名称")
    .preamble("系统提示")
    .context("上下文1")
    .context("上下文2")
    .temperature(0.7)
    .max_tokens(1000)
    .build();
```

### 使用 AgentBuilder

```rust
let model = client.completion_model("gpt-4o");

let agent = AgentBuilder::new(model)
    .name("名称")
    .preamble("提示")
    .temperature(0.7)
    .build();
```

---

## 工具定义

### 简单工具模板

```rust
#[derive(Deserialize, Serialize)]
struct MyTool;

#[derive(Deserialize)]
struct MyToolArgs {
    param: String,
}

impl Tool for MyTool {
    const NAME: &'static str = "my_tool";
    type Error = anyhow::Error;
    type Args = MyToolArgs;
    type Output = String;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        serde_json::from_value(json!({
            "name": "my_tool",
            "description": "工具描述",
            "parameters": {
                "type": "object",
                "properties": {
                    "param": {
                        "type": "string",
                        "description": "参数描述"
                    }
                },
                "required": ["param"]
            }
        })).expect("Tool Definition")
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        // 实现逻辑
        Ok(format!("结果: {}", args.param))
    }
}
```

### 数值计算工具

```rust
#[derive(Deserialize)]
struct CalcArgs {
    x: f64,
    y: f64,
}

impl Tool for Calculator {
    async fn call(&self, args: Self::Args) -> Result<f64, Self::Error> {
        Ok(args.x + args.y)
    }
}
```

### 返回结构体的工具

```rust
#[derive(Serialize)]
struct Result {
    status: String,
    data: Vec<String>,
}

impl Tool for DataTool {
    type Output = Result;
    
    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        Ok(Result {
            status: "success".to_string(),
            data: vec!["item1".to_string()],
        })
    }
}
```

---

## 编排模式

### 1. Agent 作为工具

```rust
// 步骤 1: 创建子 Agent
let sub_agent = client
    .agent("gpt-4o")
    .preamble("专业角色")
    .build();

// 步骤 2: 包装成工具
struct SubAgentTool<M: CompletionModel>(Agent<M>);

impl<M: CompletionModel> Tool for SubAgentTool<M> {
    const NAME: &'static str = "sub_agent";
    type Args = SubAgentArgs;
    type Error = PromptError;
    type Output = String;
    
    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        self.0.chat(&args.text, vec![]).await
    }
}

// 步骤 3: 在主 Agent 中使用
let main_agent = client
    .agent("gpt-4o")
    .tool(SubAgentTool(sub_agent))
    .build();
```

### 2. 编排器模式

```rust
// 创建专业 Agent
let expert1 = client.agent("gpt-4o")
    .name("专家1")
    .preamble("专长描述")
    .build();

let expert2 = client.agent("gpt-4o")
    .name("专家2")
    .preamble("专长描述")
    .build();

// 创建编排器
let orchestrator = client.agent("gpt-4o")
    .name("编排器")
    .preamble("
        你协调多个专家：
        1. expert1 - 做 X
        2. expert2 - 做 Y
    ")
    .tool(expert1)
    .tool(expert2)
    .build();
```

### 3. 并行执行

```rust
use tokio::try_join;

let (result1, result2, result3) = try_join!(
    agent1.prompt("任务1"),
    agent2.prompt("任务2"),
    agent3.prompt("任务3"),
)?;
```

### 4. 顺序执行

```rust
let result1 = agent1.prompt("步骤1").await?;
let result2 = agent2.prompt(&format!("步骤2: {}", result1)).await?;
let result3 = agent3.prompt(&format!("步骤3: {}", result2)).await?;
```

---

## Pipeline 操作

### 基础 Pipeline

```rust
use rig::pipeline::{self, Op};

let pipeline = pipeline::new()
    .prompt(agent1)
    .map(|result| format!("处理: {}", result))
    .prompt(agent2);

let output = pipeline.try_call("输入").await?;
```

### 并行 Pipeline

```rust
use rig::{parallel};
use rig::pipeline::passthrough;

let parallel_pipeline = pipeline::new()
    .chain(parallel!(
        passthrough(),
        extract(agent1),
        extract(agent2)
    ))
    .map(|(original, r1, r2)| {
        format!("{} + {} + {}", original, r1, r2)
    });
```

### 条件路由

```rust
let pipeline = pipeline::new()
    .prompt(classifier_agent)
    .map_ok(|category: String| {
        match category.as_str() {
            "A" => Ok("路由A"),
            "B" => Ok("路由B"),
            _ => Err("未知类别"),
        }
    });
```

### Extractor Pipeline

```rust
use rig::pipeline::agent_ops::extract;

#[derive(Deserialize, JsonSchema, Serialize)]
struct Data {
    field: String,
}

let extractor = client
    .extractor::<Data>("gpt-4o")
    .preamble("提取结构化数据")
    .build();

let pipeline = pipeline::new()
    .chain(extract(extractor));
```

---

## 常用配置

### 温度设置

```rust
// 事实性任务
.temperature(0.2)

// 一般对话
.temperature(0.7)

// 创意任务
.temperature(0.9)
```

### Token 限制

```rust
.max_tokens(500)    // 简短回复
.max_tokens(2000)   // 详细回复
.max_tokens(4000)   // 长文本
```

### Multi-turn 设置

```rust
// 简单工具调用
.multi_turn(5)

// 复杂编排
.multi_turn(15)

// 非常复杂的任务
.multi_turn(30)
```

---

## 提示 Agent

### 简单提示

```rust
let response = agent.prompt("问题").await?;
```

### 带历史的对话

```rust
use rig::completion::Chat;
use rig::message::Message;

let mut history: Vec<Message> = Vec::new();

let response = agent
    .chat("消息", history.clone())
    .await?;

// 更新历史
history.push(Message::User {
    content: "消息".into(),
});
history.push(Message::Assistant {
    content: response.clone().into(),
    tool_calls: vec![],
});
```

### Multi-turn

```rust
let response = agent
    .prompt("问题")
    .multi_turn(10)
    .await?;
```

### 带详细信息

```rust
let details = agent
    .prompt("问题")
    .multi_turn(10)
    .extended_details()
    .await?;

println!("{:?}", details);  // 查看所有步骤
```

### 使用历史的 Multi-turn

```rust
let mut history: Vec<Message> = Vec::new();

let response = agent
    .prompt("问题")
    .with_history(&mut history)
    .multi_turn(10)
    .await?;
```

---

## 错误处理

### 基础错误处理

```rust
match agent.prompt("问题").await {
    Ok(response) => println!("成功: {}", response),
    Err(e) => eprintln!("错误: {}", e),
}
```

### 使用 anyhow

```rust
use anyhow::{Context, Result};

async fn my_function() -> Result<String> {
    let response = agent
        .prompt("问题")
        .await
        .context("Agent 调用失败")?;
    
    Ok(response)
}
```

### 重试逻辑

```rust
async fn with_retry(agent: &Agent<impl CompletionModel>, max_retries: u32) -> Result<String> {
    for attempt in 1..=max_retries {
        match agent.prompt("问题").await {
            Ok(response) => return Ok(response),
            Err(e) if attempt < max_retries => {
                eprintln!("尝试 {} 失败，重试...", attempt);
                tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
            }
            Err(e) => return Err(e.into()),
        }
    }
    unreachable!()
}
```

---

## 调试

### 启用日志

```rust
tracing_subscriber::fmt()
    .with_max_level(tracing::Level::DEBUG)
    .with_target(false)
    .init();
```

### 打印工具调用

```rust
impl Tool for MyTool {
    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        println!("[工具] 调用参数: {:?}", args);
        let result = // ... 处理
        println!("[工具] 返回结果: {:?}", result);
        Ok(result)
    }
}
```

### 跟踪 Pipeline

```rust
let pipeline = pipeline::new()
    .prompt(agent1)
    .map(|x| {
        println!("中间结果: {}", x);
        x
    })
    .prompt(agent2);
```

---

## 常见 Preamble 模板

### 客服代表

```rust
.preamble("
    你是专业的客户服务代表。
    
    职责：
    - 礼貌、耐心地回答客户问题
    - 使用提供的工具查询信息
    - 解决客户问题或升级给人工
    
    风格：友好、专业、高效
")
```

### 技术专家

```rust
.preamble("
    你是资深技术专家。
    
    职责：
    - 提供准确的技术信息
    - 解释复杂概念
    - 提供代码示例
    
    风格：精确、详细、实用
")
```

### 创意助手

```rust
.preamble("
    你是创意专家。
    
    职责：
    - 生成新颖的想法
    - 跳出常规思维
    - 提供多个选择
    
    风格：创新、开放、激励
")
```

### 数据分析师

```rust
.preamble("
    你是数据分析专家。
    
    职责：
    - 分析数据模式和趋势
    - 提供洞察和建议
    - 可视化数据发现
    
    风格：客观、严谨、洞察
")
```

### 编排协调员

```rust
.preamble("
    你是项目协调员，管理专业团队。
    
    可用工具：
    1. tool1 - 用于 X
    2. tool2 - 用于 Y
    
    工作流程：
    1. 分析任务需求
    2. 选择合适的工具/专家
    3. 整合结果
    4. 提供综合报告
    
    风格：系统化、高效、全面
")
```

---

## 性能优化技巧

### 1. 缓存频繁请求

```rust
use std::collections::HashMap;
use tokio::sync::RwLock;

struct CachedAgent {
    agent: Agent<M>,
    cache: Arc<RwLock<HashMap<String, String>>>,
}
```

### 2. 批量处理

```rust
let results: Vec<_> = futures::future::try_join_all(
    items.iter().map(|item| agent.prompt(item))
).await?;
```

### 3. 限制并发

```rust
use futures::stream::{self, StreamExt};

let results: Vec<_> = stream::iter(items)
    .map(|item| agent.prompt(item))
    .buffer_unordered(5)  // 最多 5 个并发
    .collect()
    .await;
```

---

## 测试模板

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_agent_basic() {
        let client = Client::from_env();
        let agent = client.agent("gpt-4o").build();
        
        let response = agent.prompt("测试").await;
        assert!(response.is_ok());
    }

    #[tokio::test]
    async fn test_agent_with_tool() {
        let agent = create_test_agent();
        
        let response = agent
            .prompt("使用工具")
            .multi_turn(5)
            .await
            .unwrap();
        
        assert!(response.contains("预期内容"));
    }
}
```

---

## 有用的类型别名

```rust
use rig::completion::CompletionModel;
use rig::agent::Agent;

type BoxedAgent = Agent<Box<dyn CompletionModel + Send + Sync>>;
type OpenAIAgent = Agent<rig::providers::openai::CompletionModel>;
```

---

## 常见问题快速解决

### Q: Agent 不调用工具？
```rust
// ✅ 使用 multi_turn
.multi_turn(5)

// ✅ 在 preamble 中明确指示
.preamble("使用提供的工具来...")
```

### Q: 如何强制使用工具？
```rust
use rig::message::ToolChoice;

.tool_choice(Some(ToolChoice::Required))
```

### Q: 如何处理流式响应？
```rust
use rig::streaming::StreamingPrompt;

let mut stream = agent.prompt_stream("问题").await?;
while let Some(chunk) = stream.next().await {
    println!("{}", chunk?);
}
```

---

**提示**: 将此文件加入书签，随时快速查找！ 🔖

