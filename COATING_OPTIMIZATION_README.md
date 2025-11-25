# 涂层性能预测及优化专家系统

## 📋 项目概述

这是一个基于 Rig 框架和 Qwen Plus 模型构建的**工业级多 Agent 协作系统**，用于涂层材料的性能预测和优化。

该系统展示了如何将复杂的工业流程通过 AI Agent 编排来实现自动化和智能化。

---

## 🎯 系统功能

### 核心功能模块

#### 1️⃣ **需求提取**（Requirement Extraction）
- 收集涂层成分信息（Al、Ti、N、X元素）
- 记录工艺参数（气压、流量、偏压、温度）
- 确认涂层结构（厚度、分层）
- 明确性能需求和应用场景
- 数据验证和合理性检查

#### 2️⃣ **性能预测**（Performance Prediction）
- **TopPhi 沉积模拟**：预测涂层形貌和微观结构
- **ML 性能预测**：基于机器学习模型预测性能指标
- **历史数据比对**：查询相似配方的历史记录
- **根因分析**：分析性能差异的原因

#### 3️⃣ **优化建议**（Optimization Suggestions）
分类优化方案：

**P1 - 成分优化**
- 调整元素配比
- 考虑元素协同效应
- 预测成分变化影响

**P2 - 结构优化**
- 多层结构设计
- 梯度/纳米结构
- 应力优化

**P3 - 工艺优化**
- 气压和流量调整
- 偏压和温度优化
- 沉积速率控制

#### 4️⃣ **迭代优化**（Iterative Optimization）
- 预测vs实测对比
- 偏差分析
- 生成试验工单
- 迭代路径决策

---

## 🏗️ 系统架构

### Agent 架构图

```
                    ┌─────────────────────┐
                    │  用户/工程师         │
                    └──────────┬──────────┘
                               │
                               ↓
                    ┌─────────────────────┐
                    │   主编排 Agent       │
                    │  (Orchestrator)     │
                    └──────────┬──────────┘
                               │
              ┌────────────────┼────────────────┐
              │                │                │
              ↓                ↓                ↓
    ┌─────────────────┐  ┌──────────────┐  ┌──────────────┐
    │ 需求提取 Agent   │  │ 性能预测     │  │ 优化建议      │
    │                 │  │ Agent        │  │ Agent Group  │
    └─────────────────┘  └──────┬───────┘  └──────┬───────┘
                                 │                 │
                         ┌───────┼─────────┐      │
                         ↓       ↓         ↓      ↓
                    ┌────────┐ ┌────┐ ┌────┐  ┌──────┐
                    │TopPhi  │ │ML  │ │历史│  │P1/P2/│
                    │模拟器  │ │模型│ │数据│  │P3优化│
                    └────────┘ └────┘ └────┘  └──────┘
                                 │
                                 ↓
                    ┌─────────────────────┐
                    │  迭代优化 Agent      │
                    │  (Iteration Mgr)    │
                    └──────────┬──────────┘
                               │
                               ↓
                    ┌─────────────────────┐
                    │  实验数据读取器      │
                    └─────────────────────┘
```

### 工作流程

```
开始
  ↓
┌─────────────────────┐
│ 阶段1: 需求提取      │
│ - 收集参数          │
│ - 验证完整性        │
└──────────┬──────────┘
           ↓
┌─────────────────────┐
│ 阶段2: 性能预测      │
│ - TopPhi 模拟       │
│ - ML 预测           │
│ - 历史对比          │
│ - 根因分析          │
└──────────┬──────────┘
           ↓
┌─────────────────────┐
│ 阶段3: 优化建议      │
│ - P1: 成分优化      │
│ - P2: 结构优化      │
│ - P3: 工艺优化      │
└──────────┬──────────┘
           ↓
┌─────────────────────┐
│ 阶段4: 迭代优化      │
│ - 生成试验工单      │
│ - 实验验证          │
│ - 数据对比          │
│ - 决策下一步        │
└──────────┬──────────┘
           ↓
    性能达标？
    ├─ 是 → 结束
    └─ 否 → 返回阶段3
```

---

## 💻 技术实现

### Agent 组成

1. **需求提取专家** (`requirement_agent`)
   - 角色：数据收集和验证
   - 温度：0.3（精确）
   - 无工具

2. **性能预测专家** (`prediction_agent`)
   - 角色：多维度性能预测
   - 温度：0.3（精确）
   - 工具：TopPhi模拟器、ML预测器、历史数据查询

3. **成分优化专家** (`composition_optimizer`)
   - 角色：P1优化
   - 温度：0.4（平衡）
   - 无工具

4. **结构优化专家** (`structure_optimizer`)
   - 角色：P2优化
   - 温度：0.5（创新）
   - 无工具

5. **工艺优化专家** (`process_optimizer`)
   - 角色：P3优化
   - 温度：0.4（实用）
   - 无工具

6. **迭代优化管理专家** (`iteration_agent`)
   - 角色：流程管理和决策
   - 温度：0.3（系统化）
   - 工具：实验数据读取器

7. **系统总控** (`orchestrator`)
   - 角色：协调所有专家
   - 温度：0.5（综合）
   - 工具：所有6个专业Agent

### 模拟工具

由于是演示系统，所有专业工具都通过模拟实现：

1. **TopPhiSimulator** - 沉积形貌模拟
2. **MLPerformancePredictor** - ML性能预测
3. **HistoricalDataQuery** - 历史数据查询
4. **ExperimentalDataReader** - 实验数据读取

在实际部署时，这些可以替换为真实的工具接口。

---

## 🚀 运行示例

### 环境要求

```bash
# 设置 Qwen API 密钥
export QWEN_API_KEY="your-api-key-here"
```

### 运行命令

```bash
cargo run --example coating_optimization_system
```

### 预期输出

系统会按照以下流程执行：

1. **初始化所有 Agent**
```
✓ 需求提取专家 - 就绪
✓ 性能预测专家 - 就绪
✓ 成分优化专家 - 就绪
✓ 结构优化专家 - 就绪
✓ 工艺优化专家 - 就绪
✓ 迭代优化管理专家 - 就绪
✓ 系统总控 - 就绪
```

2. **接收用户需求**
```
当前方案：
- 成分: Al 50%, Ti 40%, N 10%
- 工艺: 气压0.6 Pa, 偏压90 V, 温度550°C
- 结构: 单层，厚度 3 μm

目标性能：
- 硬度 ≥ 3500 HV
- 附着力 ≥ 70 N
```

3. **执行预测和分析**
```
[TopPhi模拟器] 开始模拟涂层沉积...
  ✓ 模拟完成
  
[ML性能预测器] 使用机器学习模型预测性能...
  ✓ 预测完成
  - 硬度: 3250 HV (置信度: 92%)
  - 附着力: 68.5 N (置信度: 88%)
  
[历史数据库] 查询相似配方...
  ✓ 找到 5 条相似记录
```

4. **生成优化建议**

系统会调用相应的优化专家，生成P1/P2/P3优化方案。

5. **迭代优化**

在实验验证后，系统会读取实测数据并进行对比分析。

---

## 📊 数据结构

### 输入数据

```rust
// 涂层成分
CoatingComposition {
    al: 0.50,   // 50%
    ti: 0.40,   // 40%
    n: 0.10,    // 10%
    x: 0.0,     // 其他元素
}

// 工艺参数
ProcessParameters {
    n2_flow: 210.0,       // N2流量 sccm
    ar_flow: 280.0,       // Ar流量 sccm
    kr_flow: 200.0,       // Kr流量 sccm
    pressure: 0.6,        // 气压 Pa
    bias_voltage: 90.0,   // 偏压 V
    temperature: 550.0,   // 温度 °C
}

// 涂层结构
CoatingStructure {
    total_thickness: 3.0,        // 总厚度 μm
    layer_ratios: vec![1.0],     // 单层
    layer_descriptions: vec!["TiAlN"],
}
```

### 输出数据

```rust
// 预测结果
PredictionResult {
    predicted_hardness: 3250.0,
    predicted_adhesion: 68.5,
    structure_morphology: "柱状晶...",
    confidence_score: 0.92,
    historical_comparison: "...",
}

// 优化建议
OptimizationSuggestion {
    category: "P1-成分优化",
    suggestions: vec![
        "增加Al含量至55%以提高硬度",
        "优化Ti/Al比以改善韧性",
    ],
    expected_improvement: 8.5,
    priority: 1,
}
```

---

## 🎨 编排模式亮点

### 1. **层级 Agent 编排**

```rust
主编排器
  ├─ 需求提取 Agent
  ├─ 性能预测 Agent
  │   ├─ TopPhi工具
  │   ├─ ML预测工具
  │   └─ 历史数据工具
  ├─ 成分优化 Agent
  ├─ 结构优化 Agent
  ├─ 工艺优化 Agent
  └─ 迭代优化 Agent
      └─ 实验数据工具
```

### 2. **Agent-as-Tool 模式**

所有专业 Agent 都被包装成主编排器的工具：

```rust
let orchestrator = AgentBuilder::new(model)
    .tool(requirement_agent)      // Agent作为工具
    .tool(prediction_agent)       // Agent作为工具
    .tool(composition_optimizer)  // Agent作为工具
    .tool(structure_optimizer)    // Agent作为工具
    .tool(process_optimizer)      // Agent作为工具
    .tool(iteration_agent)        // Agent作为工具
    .build();
```

### 3. **工具链式调用**

性能预测 Agent 使用多个工具完成任务：

```rust
let prediction_agent = AgentBuilder::new(model)
    .tool(TopPhiSimulator)          // 工具1
    .tool(MLPerformancePredictor)   // 工具2
    .tool(HistoricalDataQuery)      // 工具3
    .build();
```

### 4. **Multi-turn 对话**

允许 Agent 多次调用工具完成复杂任务：

```rust
orchestrator
    .prompt(request)
    .multi_turn(25)  // 最多25轮交互
    .await?
```

---

## 🔧 定制和扩展

### 替换真实工具

将模拟工具替换为实际接口：

```rust
// 示例：连接真实的TopPhi模拟软件
impl Tool for TopPhiSimulator {
    async fn call(&self, args: Self::Args) -> Result<Self::Output> {
        // 调用TopPhi API
        let client = TopPhiClient::new();
        let result = client.simulate(
            &args.composition,
            &args.process_params,
        ).await?;
        Ok(result)
    }
}
```

### 添加新的优化类别

```rust
// P4 - 成本优化 Agent
let cost_optimizer = AgentBuilder::new(model)
    .name("成本优化专家")
    .preamble("优化涂层成本...")
    .build();

// 添加到编排器
let orchestrator = orchestrator.tool(cost_optimizer);
```

### 集成数据库

```rust
// 连接真实的历史数据库
impl Tool for HistoricalDataQuery {
    async fn call(&self, args: Self::Args) -> Result<Self::Output> {
        let db = Database::connect("postgresql://...").await?;
        let results = db.query_similar_coatings(&args).await?;
        Ok(results)
    }
}
```

---

## 📈 应用场景

### 1. **研发阶段**
- 快速预测新配方性能
- 减少试错次数
- 加速配方开发

### 2. **生产优化**
- 工艺参数优化
- 质量问题根因分析
- 批次一致性改进

### 3. **知识管理**
- 历史数据挖掘
- 专家知识沉淀
- 配方数据库构建

### 4. **智能决策**
- 自动生成试验方案
- 优化路径推荐
- 性能预测评估

---

## 🎯 系统特色

### ✅ **模块化设计**
每个 Agent 职责明确，易于维护和扩展

### ✅ **专业化分工**
6个专业 Agent 分别处理不同的专业领域

### ✅ **工具集成**
支持集成各种专业工具和模型

### ✅ **流程自动化**
从需求到优化建议的全流程自动化

### ✅ **可追溯性**
完整的决策过程和数据链路

### ✅ **可扩展性**
易于添加新的优化策略和工具

---

## 📚 学习价值

这个示例展示了：

1. **复杂系统的 Agent 编排**
   - 7个 Agent 协同工作
   - 层级化的架构设计

2. **Agent-as-Tool 模式**
   - 将 Agent 作为其他 Agent 的工具
   - 实现模块化和可复用

3. **工具集成**
   - 自定义工具的实现
   - 工具链式调用

4. **工业应用实践**
   - 真实的工业场景
   - 完整的业务流程

5. **Multi-turn 对话管理**
   - 复杂任务的多轮交互
   - 上下文管理

---

## 🔗 相关资源

- [Rig Agent 主教程](../rig_agent_tutorial.md)
- [Agent 编排模式](../AGENT_TUTORIAL_README.md)
- [快速参考手册](../AGENT_CHEATSHEET.md)
- [Qwen API 文档](https://help.aliyun.com/zh/dashscope/)

---

## 💡 实际部署建议

### 1. **工具实现**
- 连接真实的模拟软件（TopPhi等）
- 部署ML模型服务
- 构建历史数据库

### 2. **性能优化**
- 缓存常用查询结果
- 并行执行独立任务
- 优化 Agent 提示词

### 3. **错误处理**
- 添加重试机制
- 验证工具输出
- 异常情况处理

### 4. **用户界面**
- Web界面集成
- 可视化展示
- 交互式参数调整

### 5. **权限管理**
- 用户认证
- 操作审计
- 数据安全

---

## 🎓 总结

这个涂层优化系统展示了如何将 Rig 框架应用于**复杂的工业场景**：

- ✅ 多个专业 Agent 的协作
- ✅ Agent-as-Tool 编排模式
- ✅ 工具链集成
- ✅ 完整的业务流程自动化
- ✅ 可扩展的架构设计

这是一个**生产级示例**，可以作为其他工业AI应用的参考架构。

---

**开始使用**: `cargo run --example coating_optimization_system`

**技术支持**: 参考主教程文档获取更多编排模式和最佳实践

