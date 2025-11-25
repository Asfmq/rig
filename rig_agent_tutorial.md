# Rig 框架 Agent 教程：从基础到编排

本教程将深入介绍 Rig 框架中的 Agent 系统，特别关注 Agent 的创建、配置和编排模式。

## 目录

1. [Agent 基础概念](#1-agent-基础概念)
2. [创建第一个 Agent](#2-创建第一个-agent)
3. [Agent 配置选项](#3-agent-配置选项)
4. [Agent 工具系统](#4-agent-工具系统)
5. [Agent 编排模式](#5-agent-编排模式)
6. [高级编排技术](#6-高级编排技术)
7. [实战案例](#7-实战案例)
8. [最佳实践](#8-最佳实践)

---

## 1. Agent 基础概念

### 什么是 Agent？

在 Rig 框架中，`Agent` 是一个结合了大型语言模型（LLM）、系统提示（preamble）、上下文文档和工具的智能实体。Agent 可以：

- **理解并执行任务**：通过自然语言与用户交互
- **使用工具**：调用函数、API 或其他 Agent
- **访问知识**：通过静态或动态上下文获取信息
- **协作**：多个 Agent 可以相互配合完成复杂任务

### Agent 的核心组件

```rust
pub struct Agent<M: CompletionModel> {
    pub name: Option<String>,              // Agent 名称
    pub description: Option<String>,       // Agent 描述
    pub model: Arc<M>,                     // 底层 LLM 模型
    pub preamble: Option<String>,          // 系统提示
    pub static_context: Vec<Document>,     // 静态上下文文档
    pub temperature: Option<f64>,          // 温度参数
    pub max_tokens: Option<u64>,           // 最大 token 数
    pub additional_params: Option<Value>,  // 额外参数
    pub tool_server_handle: ToolServerHandle, // 工具服务器
    pub dynamic_context: DynamicContextStore, // 动态上下文（RAG）
    pub tool_choice: Option<ToolChoice>,   // 工具选择策略
}
```

---

## 2. 创建第一个 Agent

### 2.1 最简单的 Agent

```rust
use rig::prelude::*;
use rig::completion::Prompt;
use rig::providers::openai;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    // 创建 OpenAI 客户端
    let client = openai::Client::from_env();

    // 创建一个简单的 Agent
    let comedian_agent = client
        .agent("gpt-4o")
        .preamble("你是一位幽默的喜剧演员，擅长使用笑话和幽默来娱乐用户。")
        .build();

    // 提示 Agent 并获取响应
    let response = comedian_agent.prompt("给我讲个笑话！").await?;

    println!("{}", response);

    Ok(())
}
```

### 2.2 使用 AgentBuilder

对于更复杂的场景，可以使用 `AgentBuilder`：

```rust
use rig::agent::AgentBuilder;
use rig::providers::openai::Client;

let client = Client::from_env();
let model = client.completion_model("gpt-4o");

let agent = AgentBuilder::new(model)
    .name("助理")
    .description("通用助理，可以回答各种问题")
    .preamble("你是一个乐于助人的 AI 助理。")
    .temperature(0.7)
    .max_tokens(2000)
    .build();
```

---

## 3. Agent 配置选项

### 3.1 系统提示（Preamble）

系统提示定义了 Agent 的行为和个性：

```rust
let agent = client
    .agent("gpt-4o")
    .preamble("
        你是一位专业的技术文档编写专家。
        
        你的职责：
        1. 编写清晰、准确的技术文档
        2. 使用适当的格式和结构
        3. 为代码示例提供详细说明
        4. 确保文档易于理解
        
        写作风格：
        - 使用简洁的语言
        - 避免行话（除非必要）
        - 提供实际例子
        - 保持专业但友好的语气
    ")
    .build();
```

### 3.2 静态上下文

为 Agent 提供始终可用的背景信息：

```rust
let agent = client
    .agent("gpt-4o")
    .preamble("你是一个客服代表。")
    .context("公司名称：技术创新有限公司")
    .context("营业时间：周一至周五 9:00-18:00")
    .context("退货政策：30天内无理由退货")
    .build();
```

### 3.3 温度和参数调整

```rust
let creative_agent = client
    .agent("gpt-4o")
    .temperature(0.9)  // 高温度 = 更有创意
    .max_tokens(1500)
    .build();

let analytical_agent = client
    .agent("gpt-4o")
    .temperature(0.2)  // 低温度 = 更确定性
    .max_tokens(1000)
    .build();
```

---

## 4. Agent 工具系统

### 4.1 创建自定义工具

工具允许 Agent 执行特定的操作：

```rust
use rig::tool::Tool;
use rig::completion::ToolDefinition;
use serde::{Deserialize, Serialize};
use serde_json::json;

// 定义工具参数
#[derive(Deserialize)]
struct CalculatorArgs {
    x: f64,
    y: f64,
    operation: String,
}

// 定义工具
#[derive(Deserialize, Serialize)]
struct Calculator;

impl Tool for Calculator {
    const NAME: &'static str = "calculator";
    type Error = anyhow::Error;
    type Args = CalculatorArgs;
    type Output = f64;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        serde_json::from_value(json!({
            "name": "calculator",
            "description": "执行基本的数学运算",
            "parameters": {
                "type": "object",
                "properties": {
                    "x": {
                        "type": "number",
                        "description": "第一个数字"
                    },
                    "y": {
                        "type": "number",
                        "description": "第二个数字"
                    },
                    "operation": {
                        "type": "string",
                        "enum": ["add", "subtract", "multiply", "divide"],
                        "description": "要执行的操作"
                    }
                },
                "required": ["x", "y", "operation"]
            }
        })).expect("Tool Definition")
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let result = match args.operation.as_str() {
            "add" => args.x + args.y,
            "subtract" => args.x - args.y,
            "multiply" => args.x * args.y,
            "divide" => {
                if args.y == 0.0 {
                    return Err(anyhow::anyhow!("除数不能为零"));
                }
                args.x / args.y
            },
            _ => return Err(anyhow::anyhow!("未知操作")),
        };
        
        println!("执行 {} {} {} = {}", args.x, args.operation, args.y, result);
        Ok(result)
    }
}
```

### 4.2 为 Agent 添加工具

```rust
let calculator_agent = client
    .agent("gpt-4o")
    .preamble("你是一个数学助手，使用提供的工具来执行计算。")
    .tool(Calculator)
    .build();

// 使用 multi_turn 允许 Agent 多次调用工具
let response = calculator_agent
    .prompt("计算 (15 + 25) * 3，然后除以 2")
    .multi_turn(10)
    .await?;

println!("结果: {}", response);
```

### 4.3 多个工具

```rust
#[derive(Deserialize, Serialize)]
struct WeatherTool;

#[derive(Deserialize, Serialize)]
struct DatabaseTool;

let multi_tool_agent = client
    .agent("gpt-4o")
    .preamble("你可以访问多个工具来帮助用户。")
    .tool(Calculator)
    .tool(WeatherTool)
    .tool(DatabaseTool)
    .build();
```

---

## 5. Agent 编排模式

### 5.1 模式一：Agent 作为工具（Agent-as-Tool）

最强大的编排模式之一是将一个 Agent 作为另一个 Agent 的工具：

```rust
use rig::agent::{Agent, AgentBuilder};
use rig::tool::Tool;
use rig::completion::{CompletionModel, ToolDefinition};

// 步骤 1: 创建专门的翻译 Agent
let translator_agent = client
    .agent("gpt-4o")
    .preamble("
        你是一位专业的翻译专家。
        将任何输入的文本翻译成英文。
        如果已经是英文，则修正语法和拼写错误。
    ")
    .build();

// 步骤 2: 将 Agent 包装成工具
struct TranslatorTool<M: CompletionModel>(Agent<M>);

#[derive(Deserialize)]
struct TranslatorArgs {
    text: String,
}

impl<M: CompletionModel> Tool for TranslatorTool<M> {
    const NAME: &'static str = "translator";
    type Args = TranslatorArgs;
    type Error = rig::completion::PromptError;
    type Output = String;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        serde_json::from_value(json!({
            "name": "translator",
            "description": "将文本翻译成英文或修正英文语法错误",
            "parameters": {
                "type": "object",
                "properties": {
                    "text": {
                        "type": "string",
                        "description": "要翻译的文本"
                    }
                },
                "required": ["text"]
            }
        })).expect("Tool Definition")
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        // 调用翻译 Agent
        let result = self.0.chat(&args.text, vec![]).await?;
        println!("翻译结果: {}", result);
        Ok(result)
    }
}

// 步骤 3: 创建使用翻译工具的主 Agent
let translator_tool = TranslatorTool(translator_agent);

let main_agent = client
    .agent("gpt-4o")
    .preamble(&format!(
        "你是一个多语言助理。
        当收到非英文输入时，使用 {} 工具先翻译成英文。
        然后用英文回答用户的问题。",
        translator_tool.name()
    ))
    .tool(translator_tool)
    .build();
```

**使用场景：**
- 委派专门任务给专业 Agent
- 模块化复杂系统
- 重用 Agent 功能

### 5.2 模式二：编排器模式（Orchestrator Pattern）

创建一个主 Agent 来协调多个专业 Agent：

```rust
// 创建专业 Agent
let research_agent = client
    .agent("gpt-4o")
    .name("研究专员")
    .preamble("你是环境科学和可持续性方面的研究专家。")
    .build();

let analysis_agent = client
    .agent("gpt-4o")
    .name("数据分析师")
    .preamble("你专门解读环境和可持续性数据。")
    .build();

let recommendation_agent = client
    .agent("gpt-4o")
    .name("建议顾问")
    .preamble("你提供实用的可持续性解决方案建议。")
    .build();

// 创建编排器 Agent
let orchestrator = client
    .agent("gpt-4o")
    .name("主编排器")
    .preamble("
        你是一位环境可持续性顾问，协调多个专业团队。
        
        可用工具：
        1. research_agent - 提供详细的环境科学信息
        2. analysis_agent - 解读环境数据和统计
        3. recommendation_agent - 生成实用的可持续性解决方案
        
        工作流程：
        1. 使用 research_agent 收集背景信息
        2. 使用 analysis_agent 分析相关数据
        3. 使用 recommendation_agent 生成行动建议
        
        整合所有信息，为用户提供全面、准确的建议。
    ")
    .tool(research_agent)
    .tool(analysis_agent)
    .tool(recommendation_agent)
    .build();

// 使用编排器
let response = orchestrator
    .prompt("我的公司如何减少碳足迹？")
    .multi_turn(15)
    .await?;
```

### 5.3 模式三：路由模式（Router Pattern）

根据输入类型路由到不同的 Agent：

```rust
use rig::pipeline::{self, Op, TryOp};

// 创建分类器 Agent
let classifier_agent = client
    .agent("gpt-4o")
    .preamble("
        分类用户的请求到以下类别之一：
        - technical: 技术问题
        - business: 商业咨询
        - creative: 创意任务
        
        只返回类别名称。
    ")
    .build();

// 创建专门的 Agent
let technical_agent = client
    .agent("gpt-4o")
    .preamble("你是技术专家。")
    .build();

let business_agent = client
    .agent("gpt-4o")
    .preamble("你是商业顾问。")
    .build();

let creative_agent = client
    .agent("gpt-4o")
    .preamble("你是创意专家。")
    .build();

// 创建路由 Pipeline
let router_pipeline = pipeline::new()
    .prompt(classifier_agent)
    .map_ok(|category: String| {
        match category.trim() {
            "technical" => Ok("技术问题已路由"),
            "business" => Ok("商业问题已路由"),
            "creative" => Ok("创意任务已路由"),
            _ => Err(format!("未知类别: {}", category))
        }
    })
    .map(|x| x.unwrap().unwrap());

let response = router_pipeline
    .try_call("如何优化我们的销售流程？")
    .await?;
```

### 5.4 模式四：并行执行模式（Parallel Pattern）

同时运行多个 Agent 并整合结果：

```rust
use rig::pipeline::{self, Op, passthrough};
use rig::pipeline::agent_ops::extract;
use rig::{parallel};
use schemars::JsonSchema;

// 定义评分结构
#[derive(serde::Deserialize, JsonSchema, serde::Serialize)]
struct Score {
    score: f32,
    reasoning: String,
}

// 创建多个评估 Agent
let sentiment_agent = client
    .extractor::<Score>("gpt-4o")
    .preamble("分析文本的情感得分（0-1）。")
    .build();

let quality_agent = client
    .extractor::<Score>("gpt-4o")
    .preamble("评估文本质量得分（0-1）。")
    .build();

let relevance_agent = client
    .extractor::<Score>("gpt-4o")
    .preamble("评估文本相关性得分（0-1）。")
    .build();

// 并行执行
let parallel_pipeline = pipeline::new()
    .chain(parallel!(
        passthrough(),
        extract(sentiment_agent),
        extract(quality_agent),
        extract(relevance_agent)
    ))
    .map(|(text, sentiment, quality, relevance)| {
        format!(
            "原文: {}\n情感得分: {:.2}\n质量得分: {:.2}\n相关性得分: {:.2}",
            text,
            sentiment.unwrap().score,
            quality.unwrap().score,
            relevance.unwrap().score
        )
    });

let result = parallel_pipeline
    .call("这是一篇优秀的技术文章。")
    .await;
```

### 5.5 模式五：顺序链式模式（Sequential Chain Pattern）

任务按顺序通过多个 Agent 处理：

```rust
// Agent 1: 内容生成器
let content_generator = client
    .agent("gpt-4o")
    .preamble("根据主题生成原始内容。")
    .build();

// Agent 2: 编辑器
let editor = client
    .agent("gpt-4o")
    .preamble("改进和编辑内容，修正错误。")
    .build();

// Agent 3: 优化器
let optimizer = client
    .agent("gpt-4o")
    .preamble("优化内容以提高可读性和影响力。")
    .build();

// 创建链式 Pipeline
let content_pipeline = pipeline::new()
    .prompt(content_generator)
    .prompt(editor)
    .prompt(optimizer);

let final_content = content_pipeline
    .try_call("写一篇关于 Rust 异步编程的文章")
    .await?;
```

---

## 6. 高级编排技术

### 6.1 带有向量存储的 RAG Agent

将动态知识检索与 Agent 结合：

```rust
use rig::embeddings::EmbeddingsBuilder;
use rig::vector_store::in_memory_store::InMemoryVectorStore;
use rig::Embed;

// 定义知识库条目
#[derive(Embed, Clone, Debug, serde::Deserialize, serde::Serialize)]
struct KnowledgeEntry {
    #[embed]
    content: String,
    title: String,
}

// 创建知识库
let knowledge_entries = vec![
    KnowledgeEntry {
        title: "Rust 所有权".to_string(),
        content: "Rust 的所有权系统是其内存安全的核心...".to_string(),
    },
    KnowledgeEntry {
        title: "异步编程".to_string(),
        content: "Rust 的异步编程使用 async/await 语法...".to_string(),
    },
];

// 创建嵌入模型
let embedding_model = client.embedding_model("text-embedding-ada-002");

// 构建向量存储
let embeddings = EmbeddingsBuilder::new(embedding_model.clone())
    .documents(knowledge_entries)?
    .build()
    .await?;

let vector_store = InMemoryVectorStore::from_documents(embeddings);
let vector_index = vector_store.index(embedding_model);

// 创建带有动态上下文的 Agent
let rag_agent = client
    .agent("gpt-4o")
    .preamble("你是 Rust 编程专家。使用提供的知识库回答问题。")
    .dynamic_context(3, vector_index)  // 每次检索前 3 个最相关的文档
    .build();

let response = rag_agent
    .prompt("Rust 的所有权系统如何工作？")
    .await?;
```

### 6.2 思考工具（Think Tool）

使用 Claude 的思考工具进行复杂推理：

```rust
use rig::providers::anthropic;
use rig::tools::ThinkTool;

let anthropic_client = anthropic::Client::from_env();

let thinking_agent = anthropic_client
    .agent(anthropic::CLAUDE_3_7_SONNET)
    .preamble("
        你是一个解决问题的专家。
        对于复杂问题，使用 'think' 工具逐步推理。
        在分析工具结果时，先思考再回应。
    ")
    .tool(ThinkTool)
    .build();

let response = thinking_agent
    .prompt("
        为 8 人策划晚宴，包括 2 位素食者和 1 位麸质过敏者。
        创建一份每个人都能享用的菜单，包括开胃菜、主菜和甜点。
    ")
    .multi_turn(10)
    .await?;
```

### 6.3 多轮对话与历史管理

```rust
use rig::message::Message;
use rig::completion::Chat;

let conversational_agent = client
    .agent("gpt-4o")
    .preamble("你是一个友好的助理，记住对话上下文。")
    .build();

// 维护对话历史
let mut chat_history: Vec<Message> = Vec::new();

// 第一轮
let response1 = conversational_agent
    .chat("我的名字是张三", chat_history.clone())
    .await?;
chat_history.push(Message::User {
    content: "我的名字是张三".into(),
});
chat_history.push(Message::Assistant {
    content: response1.clone().into(),
    tool_calls: vec![],
});

// 第二轮（Agent 应该记得名字）
let response2 = conversational_agent
    .chat("我叫什么名字？", chat_history.clone())
    .await?;

println!("Agent 回复: {}", response2);
```

### 6.4 自主 Agent 循环

Agent 自主决定何时完成任务：

```rust
let autonomous_agent = client
    .agent("gpt-4o")
    .preamble("
        你是一个自主任务执行者。
        
        工作流程：
        1. 分析任务需求
        2. 规划执行步骤
        3. 执行每个步骤
        4. 验证结果
        5. 如果需要，迭代改进
        
        当任务完全完成时，在回复中包含 [TASK_COMPLETE]。
    ")
    .tool(ResearchTool)
    .tool(AnalysisTool)
    .tool(ValidationTool)
    .build();

let mut iterations = 0;
let max_iterations = 20;
let mut current_prompt = "研究并总结量子计算的最新进展";

loop {
    iterations += 1;
    if iterations > max_iterations {
        println!("达到最大迭代次数");
        break;
    }

    let response = autonomous_agent
        .prompt(current_prompt)
        .multi_turn(5)
        .await?;

    println!("迭代 {}: {}", iterations, response);

    if response.contains("[TASK_COMPLETE]") {
        println!("任务完成！");
        break;
    }

    // 准备下一次迭代
    current_prompt = &format!("继续之前的任务。上次响应: {}", response);
}
```

---

## 7. 实战案例

### 案例 1: 完整的客户服务系统

```rust
use rig::prelude::*;
use rig::providers::openai::Client;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let client = Client::from_env();

    // 知识库 Agent
    let kb_agent = client
        .agent("gpt-4o")
        .name("knowledge_base")
        .preamble("你可以访问产品知识库，回答产品相关问题。")
        .context("产品 A: 智能手表，价格 ¥1999，保修 2 年")
        .context("产品 B: 无线耳机，价格 ¥599，保修 1 年")
        .build();

    // 订单处理 Agent
    let order_agent = client
        .agent("gpt-4o")
        .name("order_processor")
        .preamble("你处理订单查询和更新。")
        .tool(CheckOrderStatusTool)
        .tool(UpdateOrderTool)
        .build();

    // 退款处理 Agent
    let refund_agent = client
        .agent("gpt-4o")
        .name("refund_processor")
        .preamble("你处理退款请求，遵循 30 天退货政策。")
        .tool(ProcessRefundTool)
        .build();

    // 主客服 Agent
    let customer_service = client
        .agent("gpt-4o")
        .name("customer_service")
        .preamble("
            你是主要的客户服务代表。
            
            根据客户查询类型，使用适当的工具：
            - 产品问题: 使用 knowledge_base
            - 订单查询: 使用 order_processor
            - 退款请求: 使用 refund_processor
            
            始终保持礼貌和专业。
        ")
        .tool(kb_agent)
        .tool(order_agent)
        .tool(refund_agent)
        .build();

    // 处理客户查询
    let response = customer_service
        .prompt("我想查询订单 #12345 的状态，并了解产品 A 的详情")
        .multi_turn(10)
        .await?;

    println!("客服回复: {}", response);

    Ok(())
}
```

### 案例 2: 内容创作工作流

```rust
// 创建内容创作系统
let topic_generator = client
    .extractor::<Vec<String>>("gpt-4o")
    .preamble("生成给定主题的 5 个内容创意。")
    .build();

let content_writer = client
    .agent("gpt-4o")
    .preamble("根据创意撰写详细内容。")
    .temperature(0.8)
    .build();

let seo_optimizer = client
    .agent("gpt-4o")
    .preamble("优化内容以提高 SEO。")
    .build();

let fact_checker = client
    .agent("gpt-4o")
    .preamble("验证内容中的事实和声明。")
    .temperature(0.3)
    .build();

// 编排工作流
async fn create_content(topic: &str) -> Result<String, anyhow::Error> {
    // 1. 生成创意
    let ideas = topic_generator.extract(topic).await?;
    println!("生成的创意: {:?}", ideas);

    // 2. 为每个创意生成内容
    let mut contents = Vec::new();
    for idea in ideas.iter().take(3) {  // 只取前 3 个
        let content = content_writer.prompt(idea).await?;
        contents.push(content);
    }

    // 3. 选择最佳内容（这里简化为第一个）
    let best_content = &contents[0];

    // 4. SEO 优化
    let optimized = seo_optimizer
        .prompt(&format!("优化这篇内容: {}", best_content))
        .await?;

    // 5. 事实检查
    let verified = fact_checker
        .prompt(&format!("验证这篇内容: {}", optimized))
        .await?;

    Ok(verified)
}
```

### 案例 3: 代码审查助手

```rust
// 代码分析 Agent
let syntax_checker = client
    .agent("gpt-4o")
    .preamble("检查代码的语法问题和错误。")
    .build();

let security_analyzer = client
    .agent("gpt-4o")
    .preamble("识别代码中的安全漏洞。")
    .build();

let performance_reviewer = client
    .agent("gpt-4o")
    .preamble("分析代码性能并提供优化建议。")
    .build();

let best_practices = client
    .agent("gpt-4o")
    .preamble("检查代码是否遵循最佳实践和编码标准。")
    .build();

// 主审查 Agent
let code_reviewer = client
    .agent("gpt-4o")
    .preamble("
        你是代码审查协调员。
        
        审查流程：
        1. syntax_checker - 检查语法
        2. security_analyzer - 安全分析
        3. performance_reviewer - 性能评估
        4. best_practices - 最佳实践检查
        
        整合所有反馈，生成综合审查报告。
    ")
    .tool(syntax_checker)
    .tool(security_analyzer)
    .tool(performance_reviewer)
    .tool(best_practices)
    .build();

let code = r#"
fn process_data(data: Vec<String>) -> Vec<String> {
    let mut results = Vec::new();
    for item in data {
        results.push(item.to_uppercase());
    }
    results
}
"#;

let review = code_reviewer
    .prompt(&format!("审查这段 Rust 代码: {}", code))
    .multi_turn(10)
    .await?;
```

---

## 8. 最佳实践

### 8.1 设计原则

1. **单一职责**: 每个 Agent 应该有明确的、专注的职责
```rust
// ✅ 好 - 职责明确
let translator = client.agent("gpt-4o")
    .preamble("你只做翻译")
    .build();

// ❌ 差 - 职责过多
let swiss_army_agent = client.agent("gpt-4o")
    .preamble("你做翻译、数据分析、代码审查...")
    .build();
```

2. **明确的 Preamble**: 清晰定义 Agent 的角色、能力和限制
```rust
let good_preamble = "
    角色: 你是 Python 编程专家
    
    能力:
    - 回答 Python 相关问题
    - 提供代码示例
    - 解释 Python 概念
    
    限制:
    - 不讨论其他编程语言
    - 不提供未经测试的代码
    
    风格: 简洁、准确、实用
";
```

3. **合理的温度设置**: 根据任务类型调整
```rust
// 事实性任务 - 低温度
let factual_agent = client.agent("gpt-4o")
    .temperature(0.2)
    .build();

// 创意任务 - 高温度
let creative_agent = client.agent("gpt-4o")
    .temperature(0.9)
    .build();
```

### 8.2 错误处理

```rust
use anyhow::{Context, Result};

async fn safe_agent_call(agent: &Agent<impl CompletionModel>, prompt: &str) -> Result<String> {
    agent
        .prompt(prompt)
        .await
        .context("Agent 调用失败")?;
    
    Ok(response)
}

// 使用重试逻辑
async fn agent_call_with_retry(
    agent: &Agent<impl CompletionModel>,
    prompt: &str,
    max_retries: u32,
) -> Result<String> {
    for attempt in 1..=max_retries {
        match agent.prompt(prompt).await {
            Ok(response) => return Ok(response),
            Err(e) if attempt < max_retries => {
                eprintln!("尝试 {} 失败: {}. 重试...", attempt, e);
                tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
            }
            Err(e) => return Err(e.into()),
        }
    }
    unreachable!()
}
```

### 8.3 性能优化

1. **并行执行独立任务**
```rust
use tokio::try_join;

// 并行调用多个 Agent
let (result1, result2, result3) = try_join!(
    agent1.prompt("任务 1"),
    agent2.prompt("任务 2"),
    agent3.prompt("任务 3"),
)?;
```

2. **缓存常用响应**
```rust
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

struct CachedAgent<M: CompletionModel> {
    agent: Agent<M>,
    cache: Arc<RwLock<HashMap<String, String>>>,
}

impl<M: CompletionModel> CachedAgent<M> {
    async fn prompt_cached(&self, prompt: &str) -> Result<String> {
        // 检查缓存
        {
            let cache = self.cache.read().await;
            if let Some(cached) = cache.get(prompt) {
                return Ok(cached.clone());
            }
        }

        // 调用 Agent
        let response = self.agent.prompt(prompt).await?;

        // 存入缓存
        {
            let mut cache = self.cache.write().await;
            cache.insert(prompt.to_string(), response.clone());
        }

        Ok(response)
    }
}
```

3. **限制 Token 使用**
```rust
let efficient_agent = client
    .agent("gpt-4o")
    .max_tokens(500)  // 限制响应长度
    .preamble("请保持回答简洁。")
    .build();
```

### 8.4 可观测性

```rust
use tracing::{info, warn, error, instrument};

#[instrument(skip(agent))]
async fn monitored_agent_call(
    agent: &Agent<impl CompletionModel>,
    prompt: &str,
) -> Result<String> {
    info!("开始 Agent 调用: {}", prompt);
    
    let start = std::time::Instant::now();
    
    match agent.prompt(prompt).await {
        Ok(response) => {
            let duration = start.elapsed();
            info!("Agent 调用成功，耗时: {:?}", duration);
            Ok(response)
        }
        Err(e) => {
            error!("Agent 调用失败: {}", e);
            Err(e.into())
        }
    }
}

// 启用日志
fn setup_logging() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .init();
}
```

### 8.5 测试策略

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_agent_basic_response() {
        let client = Client::from_env();
        let agent = client
            .agent("gpt-4o")
            .preamble("你是测试助手。")
            .build();

        let response = agent.prompt("测试消息").await;
        assert!(response.is_ok());
    }

    #[tokio::test]
    async fn test_agent_with_tools() {
        let agent = create_calculator_agent();
        
        let response = agent
            .prompt("计算 2 + 2")
            .multi_turn(5)
            .await
            .unwrap();

        assert!(response.contains("4"));
    }

    // 模拟 Agent 进行单元测试
    struct MockAgent {
        responses: HashMap<String, String>,
    }

    impl MockAgent {
        fn new() -> Self {
            let mut responses = HashMap::new();
            responses.insert(
                "你好".to_string(),
                "你好！我能帮你什么？".to_string()
            );
            Self { responses }
        }

        fn prompt(&self, input: &str) -> Option<&String> {
            self.responses.get(input)
        }
    }
}
```

---

## 9. 常见问题

### Q1: 何时使用 multi_turn？

```rust
// 需要工具调用时使用 multi_turn
let response = agent
    .prompt("复杂任务需要多个工具")
    .multi_turn(10)  // 允许最多 10 轮
    .await?;

// 简单对话不需要 multi_turn
let response = agent.prompt("简单问题").await?;
```

### Q2: 如何控制 Agent 的成本？

```rust
// 1. 限制 token 数量
let cost_efficient = client.agent("gpt-4o")
    .max_tokens(300)
    .build();

// 2. 使用更便宜的模型
let cheap_agent = client.agent("gpt-3.5-turbo").build();

// 3. 缓存响应（见上文）
```

### Q3: Agent 之间如何共享状态？

```rust
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone)]
struct SharedState {
    data: Arc<RwLock<HashMap<String, String>>>,
}

// 在工具中访问共享状态
struct StatefulTool {
    state: SharedState,
}

impl Tool for StatefulTool {
    // ... 实现细节
    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let mut data = self.state.data.write().await;
        data.insert("key".to_string(), "value".to_string());
        // ...
    }
}
```

---

## 10. 总结

Rig 框架提供了强大而灵活的 Agent 系统，支持从简单的单一 Agent 到复杂的多 Agent 编排。

**关键要点：**

1. **Agent 是模块化的**: 每个 Agent 都有明确的职责
2. **工具是强大的**: Agent 可以使用工具扩展功能，包括其他 Agent
3. **编排模式多样**: 选择适合你需求的模式（编排器、路由、并行等）
4. **可组合性**: 小的、专注的 Agent 可以组合成复杂的系统
5. **可观测性重要**: 记录和监控 Agent 的行为

**下一步：**

- 实验不同的编排模式
- 构建自己的自定义工具
- 探索 RAG 和向量存储集成
- 优化性能和成本
- 为生产环境添加错误处理和监控

祝你在 Rig 框架中构建强大的 Agent 系统！

