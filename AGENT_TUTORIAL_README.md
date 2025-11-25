# Rig Agent 教程和示例

本目录包含了关于 Rig 框架中 Agent 系统的完整教程和实战示例。

## 📚 文档

### [rig_agent_tutorial.md](./rig_agent_tutorial.md)
完整的 Agent 教程文档，涵盖：

- **基础概念**: Agent 的定义、组件和架构
- **配置选项**: Preamble、上下文、温度等参数
- **工具系统**: 如何创建和使用自定义工具
- **编排模式**: 5 种主要的 Agent 编排模式
  - Agent 作为工具
  - 编排器模式
  - 路由模式
  - 并行执行
  - 顺序链式
- **高级技术**: RAG、思考工具、多轮对话、自主循环
- **实战案例**: 客户服务系统、内容创作、代码审查
- **最佳实践**: 设计原则、错误处理、性能优化、测试

## 🚀 运行示例

所有示例都需要设置环境变量：

```bash
export OPENAI_API_KEY=your_api_key_here
```

### 示例 1: 基础 Agent

```bash
cargo run --example tutorial_basic_agent
```

**学习内容**:
- 创建最简单的 Agent
- 添加静态上下文
- 调整温度参数
- 限制 token 数量

**关键代码片段**:
```rust
let agent = client
    .agent("gpt-4o")
    .preamble("你是一个友好的助手")
    .temperature(0.7)
    .build();

let response = agent.prompt("你的问题").await?;
```

### 示例 2: Agent 工具

```bash
cargo run --example tutorial_agent_with_tools
```

**学习内容**:
- 创建自定义工具
- 实现 Tool trait
- 为 Agent 添加多个工具
- 使用 multi_turn 进行工具调用

**包含的工具**:
- ✅ 计算器（加减乘除）
- ✅ 天气查询（模拟）
- ✅ 单位转换

**关键代码片段**:
```rust
impl Tool for Calculator {
    const NAME: &'static str = "calculator";
    type Args = CalculatorArgs;
    type Output = f64;

    async fn call(&self, args: Self::Args) -> Result<Self::Output> {
        // 实现工具逻辑
    }
}

let agent = client
    .agent("gpt-4o")
    .tool(Calculator)
    .build();
```

### 示例 3: Agent 编排

```bash
cargo run --example tutorial_agent_orchestration
```

**学习内容**:
- 将 Agent 作为工具使用
- 编排器模式实现
- 专业团队协作
- 多层 Agent 架构

**包含的模式**:
1. **Agent-as-Tool**: 翻译 Agent → 主 Agent
2. **编排器**: 研究 + 分析 + 总结 Agent
3. **专业团队**: 产品 + 订单 + 退款专家

**关键代码片段**:
```rust
// 将 Agent 包装成工具
struct TranslatorTool<M: CompletionModel>(Agent<M>);

impl<M: CompletionModel> Tool for TranslatorTool<M> {
    async fn call(&self, args: Self::Args) -> Result<Self::Output> {
        self.0.chat(&args.text, vec![]).await
    }
}

// 在主 Agent 中使用
let orchestrator = client
    .agent("gpt-4o")
    .tool(translator_agent)
    .tool(research_agent)
    .build();
```

### 示例 4: 高级编排

```bash
cargo run --example tutorial_advanced_orchestration
```

**学习内容**:
- Pipeline 链式处理
- 并行执行多个 Agent
- 路由模式实现
- 复杂工作流设计

**包含的模式**:
1. **Pipeline Chain**: 创意 → 撰写 → 评估
2. **Parallel**: 同时进行质量、情感、SEO 评估
3. **Router**: 根据类别路由到不同的专业 Agent
4. **Complex Workflow**: 完整的文章发布流程

**关键代码片段**:
```rust
// Pipeline 链
let pipeline = pipeline::new()
    .prompt(agent1)
    .map(|result| format!("处理: {}", result))
    .prompt(agent2);

// 并行执行
let parallel_pipeline = pipeline::new()
    .chain(parallel!(
        passthrough(),
        extract(quality_agent),
        extract(sentiment_agent),
        extract(seo_agent)
    ));
```

## 📖 快速开始指南

### 1. 创建你的第一个 Agent

```rust
use rig::prelude::*;
use rig::completion::Prompt;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let client = rig::providers::openai::Client::from_env();
    
    let agent = client
        .agent("gpt-4o")
        .preamble("定义 Agent 的角色和行为")
        .build();
    
    let response = agent.prompt("你的问题").await?;
    println!("{}", response);
    
    Ok(())
}
```

### 2. 添加工具能力

```rust
// 1. 定义工具
#[derive(Deserialize, Serialize)]
struct MyTool;

impl Tool for MyTool {
    const NAME: &'static str = "my_tool";
    // ... 实现其他方法
}

// 2. 添加到 Agent
let agent = client
    .agent("gpt-4o")
    .tool(MyTool)
    .build();

// 3. 使用 multi_turn 允许工具调用
let response = agent
    .prompt("使用工具执行任务")
    .multi_turn(5)
    .await?;
```

### 3. 创建多 Agent 系统

```rust
// 创建专业 Agent
let expert1 = client.agent("gpt-4o")
    .preamble("专家 1 的角色")
    .build();

let expert2 = client.agent("gpt-4o")
    .preamble("专家 2 的角色")
    .build();

// 创建编排器
let orchestrator = client.agent("gpt-4o")
    .preamble("协调专家的角色")
    .tool(expert1)
    .tool(expert2)
    .build();
```

## 🎯 学习路径

### 初级（Day 1-2）
1. ✅ 阅读教程 1-3 节（基础概念、创建 Agent、配置）
2. ✅ 运行 `tutorial_basic_agent.rs`
3. ✅ 修改示例，尝试不同的 preamble 和参数
4. ✅ 创建自己的简单 Agent

### 中级（Day 3-5）
1. ✅ 阅读教程第 4 节（工具系统）
2. ✅ 运行 `tutorial_agent_with_tools.rs`
3. ✅ 创建自己的自定义工具
4. ✅ 实现一个带工具的实用 Agent

### 高级（Week 2）
1. ✅ 阅读教程第 5-6 节（编排模式）
2. ✅ 运行 `tutorial_agent_orchestration.rs`
3. ✅ 运行 `tutorial_advanced_orchestration.rs`
4. ✅ 设计并实现多 Agent 系统
5. ✅ 探索 Pipeline 和并行执行

### 专家级（Week 3+）
1. ✅ 阅读教程第 7-8 节（实战案例、最佳实践）
2. ✅ 研究项目中的实际示例
3. ✅ 集成 RAG 和向量存储
4. ✅ 构建生产级 Agent 系统

## 💡 常见用例

### 客户服务机器人
```rust
let customer_service = client
    .agent("gpt-4o")
    .preamble("你是客服代表...")
    .context("产品信息...")
    .context("服务政策...")
    .tool(order_lookup_tool)
    .tool(refund_tool)
    .build();
```

### 内容创作助手
```rust
let content_writer = client
    .agent("gpt-4o")
    .preamble("你是内容创作专家...")
    .temperature(0.8)  // 更有创意
    .tool(research_tool)
    .tool(seo_tool)
    .build();
```

### 代码审查助手
```rust
let code_reviewer = client
    .agent("gpt-4o")
    .preamble("你是代码审查专家...")
    .temperature(0.3)  // 更确定性
    .tool(syntax_checker)
    .tool(security_analyzer)
    .build();
```

### 数据分析助手
```rust
let data_analyst = client
    .agent("gpt-4o")
    .preamble("你是数据分析专家...")
    .tool(query_database_tool)
    .tool(visualization_tool)
    .build();
```

## 🔧 调试技巧

### 启用日志
```rust
tracing_subscriber::fmt()
    .with_max_level(tracing::Level::DEBUG)
    .with_target(false)
    .init();
```

### 查看工具调用
所有示例都会打印工具调用信息：
```
[工具名称] 执行操作: 参数详情
[工具名称] 结果: 返回值
```

### 检查 multi_turn 过程
使用 `extended_details()` 获取详细的执行信息：
```rust
let result = agent
    .prompt("问题")
    .multi_turn(5)
    .extended_details()
    .await?;

println!("{:?}", result);  // 包含所有中间步骤
```

## 📊 性能优化

### 1. 并行执行独立任务
```rust
use tokio::try_join;

let (r1, r2, r3) = try_join!(
    agent1.prompt("任务1"),
    agent2.prompt("任务2"),
    agent3.prompt("任务3"),
)?;
```

### 2. 限制 Token 使用
```rust
let agent = client
    .agent("gpt-4o")
    .max_tokens(500)  // 限制响应长度
    .build();
```

### 3. 选择合适的温度
- **低温度 (0.1-0.3)**: 事实性任务、数据分析
- **中温度 (0.5-0.7)**: 一般对话、问答
- **高温度 (0.8-1.0)**: 创意任务、头脑风暴

## 🐛 故障排除

### Agent 没有调用工具？
- ✅ 确保使用 `.multi_turn(n)`
- ✅ 检查 preamble 是否明确指示使用工具
- ✅ 工具描述是否清晰

### 工具调用失败？
- ✅ 检查工具的 `call` 方法实现
- ✅ 验证参数类型和必需字段
- ✅ 查看错误日志

### 响应质量不佳？
- ✅ 优化 preamble，更具体地描述角色
- ✅ 调整温度参数
- ✅ 提供更多上下文
- ✅ 使用更强大的模型

## 🔗 相关资源

- [Rig 官方文档](https://github.com/0xPlaygrounds/rig)
- [rig-core 示例目录](./rig-core/examples/)
- [OpenAI API 文档](https://platform.openai.com/docs)
- [Anthropic API 文档](https://docs.anthropic.com/)

## 🤝 贡献

如果你发现示例中的问题或有改进建议：

1. 提交 Issue 描述问题
2. 提交 Pull Request 包含：
   - 清晰的问题描述
   - 修复或改进的代码
   - 更新的文档

## 📝 许可证

本教程和示例代码遵循与 Rig 项目相同的许可证。

---

**开始构建你的 Agent 系统吧！** 🚀

如有问题，请查阅完整教程文档或运行示例代码来学习。

