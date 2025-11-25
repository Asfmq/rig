# 涂层优化系统信息传递分析

## 当前工作流的信息传递方式

### 📊 现状分析

当前 `coating_optimization_system_streaming.rs` 中的信息传递方式如下：

#### 当前模式：**共享输入模式（Shared Input Pattern）**

```
用户输入 (user_request)
    ↓
    ├─→ [阶段一：需求提取 Agent] → 结果被丢弃 (_requirement_result)
    ├─→ [阶段二：性能预测 Agent] → 结果被丢弃 (_prediction_result)
    ├─→ [阶段三：成分优化 Agent] → 结果被丢弃 (_composition_result)
    ├─→ [阶段三：结构优化 Agent] → 结果被丢弃 (_structure_result)
    ├─→ [阶段三：工艺优化 Agent] → 结果被丢弃 (_process_result)
    └─→ [阶段四：迭代优化 Agent] → 结果被丢弃 (_iteration_result)
```

**特点：**
1. ✅ **独立执行**：每个 Agent 独立运行，互不依赖
2. ✅ **并行友好**：三个阶段可以并行执行
3. ❌ **信息丢失**：前一个 Agent 的输出没有传递给下一个 Agent
4. ❌ **重复输入**：每个 Agent 都接收相同的原始输入 `user_request`
5. ❌ **无上下文累积**：无法利用前面阶段的处理结果

### 🔍 代码示例分析

```rust
// 阶段一：需求提取
let _requirement_result = stream_agent_response(
    &requirement_agent, 
    &format!("请根据以下信息提取和整理涂层需求参数：\n\n{}", user_request),
    "需求提取专家"
).await?;
// ❌ 结果被丢弃，未传递给后续阶段

// 阶段二：性能预测
let _prediction_result = stream_agent_response(
    &prediction_agent,
    &format!("请基于以下参数进行多维度性能预测：\n\n{}", user_request),
    "性能预测专家"
).await?;
// ❌ 仍然使用原始 user_request，未使用阶段一的结果
```

## 🔄 改进方案

### 方案一：链式传递模式（Chained Pattern）

**优点：** 每个 Agent 基于前一个 Agent 的输出，信息逐渐精炼
**缺点：** 必须顺序执行，无法并行

```rust
// 阶段一：需求提取
let requirement_result = stream_agent_response(...).await?;

// 阶段二：性能预测（使用阶段一的结果）
let prediction_prompt = format!(
    "基于以下提取的需求参数进行性能预测：\n\n{}\n\n原始需求：\n{}",
    requirement_result,
    user_request
);
let prediction_result = stream_agent_response(...).await?;

// 阶段三：优化建议（使用阶段一和阶段二的结果）
let optimization_prompt = format!(
    "基于以下信息提出优化建议：\n\n需求参数：\n{}\n\n预测结果：\n{}\n\n原始需求：\n{}",
    requirement_result,
    prediction_result,
    user_request
);
// ...
```

### 方案二：混合模式（Hybrid Pattern）

**阶段一、二顺序执行并传递信息，阶段三并行执行但接收前面的结果**

```rust
// 阶段一：需求提取
let requirement_result = stream_agent_response(...).await?;

// 阶段二：性能预测
let prediction_result = stream_agent_response(...).await?;

// 阶段三：三个优化并行执行，但都接收前面的结果
let (comp_result, struct_result, proc_result) = tokio::try_join!(
    async {
        let prompt = format!(
            "需求参数：\n{}\n预测结果：\n{}\n请提出成分优化建议...",
            requirement_result, prediction_result
        );
        stream_agent_response(&composition_optimizer, &prompt, ...).await
    },
    async {
        let prompt = format!(
            "需求参数：\n{}\n预测结果：\n{}\n请提出结构优化建议...",
            requirement_result, prediction_result
        );
        stream_agent_response(&structure_optimizer, &prompt, ...).await
    },
    async {
        let prompt = format!(
            "需求参数：\n{}\n预测结果：\n{}\n请提出工艺优化建议...",
            requirement_result, prediction_result
        );
        stream_agent_response(&process_optimizer, &prompt, ...).await
    }
)?;
```

### 方案三：上下文累积模式（Context Accumulation Pattern）

**维护一个累积的上下文对象，每个 Agent 都接收完整上下文**

```rust
struct WorkflowContext {
    original_request: String,
    requirement_extraction: Option<String>,
    performance_prediction: Option<String>,
    composition_optimization: Option<String>,
    structure_optimization: Option<String>,
    process_optimization: Option<String>,
}

let mut ctx = WorkflowContext {
    original_request: user_request.clone(),
    ..Default::default()
};

// 阶段一
ctx.requirement_extraction = Some(
    stream_agent_response(&requirement_agent, &build_prompt(&ctx), ...).await?
);

// 阶段二
ctx.performance_prediction = Some(
    stream_agent_response(&prediction_agent, &build_prompt(&ctx), ...).await?
);

// 阶段三（并行）
let prompts = build_optimization_prompts(&ctx);
// ...
```

## 📝 建议

对于当前工作流，**推荐使用方案二（混合模式）**：

1. **阶段一和阶段二**：顺序执行，确保需求提取的结果传递给性能预测
2. **阶段三**：三个优化可以并行，但都接收阶段一和阶段二的结果
3. **阶段四**：接收前面所有阶段的结果

这样可以：
- ✅ 保持并行执行的性能优势
- ✅ 实现信息传递和上下文累积
- ✅ 每个 Agent 都能基于前面的处理结果工作

## 🔧 实现要点

1. **移除 `_` 前缀**：保存每个阶段的输出结果
2. **构建累积 prompt**：将前面的结果包含在后续 Agent 的 prompt 中
3. **合理截断**：如果结果太长，可以选择摘要或关键信息传递

