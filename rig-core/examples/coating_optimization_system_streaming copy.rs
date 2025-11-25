//! 涂层性能预测及优化专家系统（支持流式输出的手动编排版本）
//! 
//! 这个版本使用手动编排方式，而不是 agent-as-tool 模式，
//! 这样可以确保每个子 agent 的响应都能流式输出，提供更好的用户体验。

use rig::prelude::*;
use rig::agent::{Agent, AgentBuilder, stream_to_stdout};
use rig::completion::{CompletionModel, PromptError, ToolDefinition};
use rig::streaming::StreamingPrompt;
use rig::tool::Tool;
use serde::{Deserialize, Serialize};
use serde_json::json;
use futures::StreamExt;

// ============= 错误类型定义 =============

#[derive(Debug, thiserror::Error)]
enum ToolError {
    #[error("模拟工具错误: {0}")]
    SimulationError(String),
}

// ============= 模拟工具定义 =============

/// TopPhi 涂层沉积形貌模拟工具（模拟）
#[derive(Deserialize, Serialize)]
struct TopPhiSimulator;

#[derive(Deserialize)]
struct TopPhiArgs {
    composition: String,
    process_params: String,
    structure: String,
}

impl Tool for TopPhiSimulator {
    const NAME: &'static str = "topPhi_simulator";
    type Error = ToolError;
    type Args = TopPhiArgs;
    type Output = String;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        serde_json::from_value(json!({
            "name": "topPhi_simulator",
            "description": "TopPhi 模拟工具 - 预测涂层沉积形貌和微观结构",
            "parameters": {
                "type": "object",
                "properties": {
                    "composition": {"type": "string", "description": "涂层成分信息（JSON格式）"},
                    "process_params": {"type": "string", "description": "工艺参数（JSON格式）"},
                    "structure": {"type": "string", "description": "预计沉积结构（JSON格式）"}
                },
                "required": ["composition", "process_params", "structure"]
            }
        }))
        .expect("Tool Definition")
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        println!("\n[TopPhi模拟器] 开始模拟涂层沉积...");
        println!("  - 成分: {}", args.composition);
        println!("  - 工艺参数: {}", args.process_params);
        println!("  - 结构: {}", args.structure);
        
        let result = format!(
            "TopPhi模拟结果:\n\
            形貌特征: 柱状晶结构，晶粒尺寸约 50-80 nm\n\
            表面粗糙度: Ra = 0.15 μm\n\
            致密度: 98.5%\n\
            应力状态: 压应力 -2.3 GPa\n\
            界面结合: 良好，无明显缺陷\n\
            预测生长速率: 2.5 μm/h"
        );
        
        println!("  ✓ 模拟完成\n");
        Ok(result)
    }
}

/// ML 性能预测模型工具（模拟）
#[derive(Deserialize, Serialize)]
struct MLPerformancePredictor;

#[derive(Deserialize)]
struct MLPredictorArgs {
    composition: String,
    process_params: String,
    structure: String,
    simulation_result: String,
}

impl Tool for MLPerformancePredictor {
    const NAME: &'static str = "ml_performance_predictor";
    type Error = ToolError;
    type Args = MLPredictorArgs;
    type Output = String;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        serde_json::from_value(json!({
            "name": "ml_performance_predictor",
            "description": "机器学习模型 - 预测涂层性能（硬度、附着力、耐磨性等）",
            "parameters": {
                "type": "object",
                "properties": {
                    "composition": {"type": "string", "description": "涂层成分"},
                    "process_params": {"type": "string", "description": "工艺参数"},
                    "structure": {"type": "string", "description": "涂层结构"},
                    "simulation_result": {"type": "string", "description": "TopPhi模拟结果"}
                },
                "required": ["composition", "process_params", "structure", "simulation_result"]
            }
        }))
        .expect("Tool Definition")
    }

    async fn call(&self, _args: Self::Args) -> Result<Self::Output, Self::Error> {
        println!("\n[ML模型] 进行性能预测...");
        let result = format!(
            "ML模型预测结果:\n\
            预测硬度: 3200 HV (置信度: 85%)\n\
            预测附着力: 68 N (置信度: 82%)\n\
            耐磨性指数: 良好 (置信度: 78%)\n\
            热稳定性: 750°C (置信度: 80%)\n\
            综合评估: 当前方案未完全达到目标性能"
        );
        println!("  ✓ 预测完成\n");
        Ok(result)
    }
}

/// 历史数据查询工具（模拟）
#[derive(Deserialize, Serialize)]
struct HistoricalDataQuery;

#[derive(Deserialize)]
struct HistoricalQueryArgs {
    composition_range: String,
    performance_target: String,
}

impl Tool for HistoricalDataQuery {
    const NAME: &'static str = "historical_data_query";
    type Error = ToolError;
    type Args = HistoricalQueryArgs;
    type Output = String;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        serde_json::from_value(json!({
            "name": "historical_data_query",
            "description": "查询历史数据库 - 查找相似成分和工艺的实测数据",
            "parameters": {
                "type": "object",
                "properties": {
                    "composition_range": {"type": "string", "description": "成分范围"},
                    "performance_target": {"type": "string", "description": "性能目标"}
                },
                "required": ["composition_range", "performance_target"]
            }
        }))
        .expect("Tool Definition")
    }

    async fn call(&self, _args: Self::Args) -> Result<Self::Output, Self::Error> {
        println!("\n[历史数据查询] 检索相似案例...");
        let result = format!(
            "历史数据对比结果:\n\
            相似案例数: 15个\n\
            平均硬度: 3150 HV\n\
            平均附着力: 65 N\n\
            当前方案预测值略高于历史平均值\n\
            建议: 提高Al含量可能提升性能"
        );
        println!("  ✓ 查询完成\n");
        Ok(result)
    }
}

/// 实验数据读取工具（模拟）
#[derive(Deserialize, Serialize)]
struct ExperimentalDataReader;

#[derive(Deserialize)]
struct ExperimentalReaderArgs {
    sample_id: String,
}

impl Tool for ExperimentalDataReader {
    const NAME: &'static str = "experimental_data_reader";
    type Error = ToolError;
    type Args = ExperimentalReaderArgs;
    type Output = String;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        serde_json::from_value(json!({
            "name": "experimental_data_reader",
            "description": "读取实验数据 - 从实验室系统获取实际测量结果",
            "parameters": {
                "type": "object",
                "properties": {
                    "sample_id": {"type": "string", "description": "样品编号"}
                },
                "required": ["sample_id"]
            }
        }))
        .expect("Tool Definition")
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        println!("\n[实验数据读取] 读取样品 {} 的数据...", args.sample_id);
        let result = format!(
            "实验数据（样品 {}）:\n\
            实测硬度: 3250 HV\n\
            实测附着力: 69 N\n\
            磨损率: 2.5×10⁻⁶ mm³/N·m\n\
            热稳定性: 790°C\n\
            备注: 性能接近但未完全达标，建议进一步优化",
            args.sample_id
        );
        println!("  ✓ 读取完成\n");
        Ok(result)
    }
}

// ============= 辅助函数：流式调用 agent 并显示输出 =============

/// 流式调用 agent 并实时显示输出
async fn stream_agent_response<M: CompletionModel + 'static>(
    agent: &Agent<M>,
    prompt: &str,
    agent_name: &str,
) -> Result<String, PromptError>
where
    <M as CompletionModel>::StreamingResponse: Send,
{
    println!("\n【{}】开始处理...", agent_name);
    println!("{}\n", "-".repeat(60));
    print!("Response: ");
    
    let mut stream = agent.stream_prompt(prompt).multi_turn(10).await;
    
    // 使用 stream_to_stdout 处理流式输出
    let final_response = stream_to_stdout(&mut stream).await
        .map_err(|e| PromptError::CompletionError(
            rig::completion::CompletionError::ResponseError(e.to_string())
        ))?;
    
    let full_response = final_response.response().to_string();
    
    println!("\n{}\n", "-".repeat(60));
    println!("【{}】完成\n", agent_name);
    
    Ok(full_response)
}

// ============= 主函数 =============

async fn create_coating_optimization_system_with_streaming() -> Result<(), anyhow::Error> {
    // 使用 Ollama 模型
    let api_key = "sk-348d7ca647714c52aca12ea106cfa895";
    let qwen_client = rig::providers::qwen::Client::new_with_api_key(&api_key);
    let model = qwen_client.completion_model("qwen-plus");
    // let qwen_client = rig::providers::ollama::Client::new();
    // let model = qwen_client.completion_model("llama3.2");

    println!("=== 涂层性能预测及优化专家系统（流式编排版本） ===\n");
    println!("正在初始化 Agent 系统...\n");

    // 1. 需求提取 Agent
    let requirement_agent = AgentBuilder::new(model.clone())
        .name("需求提取专家")
        .preamble("
            你是涂层需求提取专家。负责收集和整理涂层成分信息（Al、Ti、N、X元素含量）、
            记录工艺参数（气压、流量、偏压、温度）、确认涂层结构信息（厚度、分层）、
            明确应用场景和性能需求，验证数据完整性和合理性。
            输出结构化的JSON格式数据。
        ")
        .temperature(0.3)
        .build();

    // 2. 性能预测 Agent
    let prediction_agent = AgentBuilder::new(model.clone())
        .name("性能预测专家")
        .preamble("
            你是涂层性能预测专家。负责调用 TopPhi 模拟器预测沉积形貌、
            使用 ML 模型预测性能指标、查询历史数据进行对比、进行根因分析、评估预测置信度。
        ")
        .tool(TopPhiSimulator)
        .tool(MLPerformancePredictor)
        .tool(HistoricalDataQuery)
        .temperature(0.3)
        .build();

    // 3. 成分优化 Agent
    let composition_optimizer = AgentBuilder::new(model.clone())
        .name("成分优化专家")
        .preamble("
            你是涂层成分优化专家（P1优化）。分析当前成分配比的优缺点、
            基于性能目标提出成分调整建议、考虑元素间协同效应、预测成分调整后的性能变化。
            输出具体的成分调整方案和理由。
        ")
        .temperature(0.4)
        .build();

    // 4. 结构优化 Agent
    let structure_optimizer = AgentBuilder::new(model.clone())
        .name("结构优化专家")
        .preamble("
            你是涂层结构优化专家（P2优化）。设计多层结构方案、优化各层厚度和占比、
            设计梯度或纳米多层结构、考虑应力释放和界面结合。
            输出详细的结构设计方案。
        ")
        .temperature(0.4)
        .build();

    // 5. 工艺优化 Agent
    let process_optimizer = AgentBuilder::new(model.clone())
        .name("工艺优化专家")
        .preamble("
            你是涂层工艺优化专家（P3优化）。优化沉积工艺参数、调整气体流量比例、
            优化偏压和温度、预测工艺参数对性能的影响。
            输出具体的工艺优化方案。
        ")
        .temperature(0.4)
        .build();

    // 6. 迭代优化 Agent
    let iteration_agent = AgentBuilder::new(model.clone())
        .name("迭代优化管理专家")
        .preamble("
            你是迭代优化流程管理专家。管理优化迭代流程、比对预测值与实测值、
            分析偏差原因、决定下一步优化方向、生成试验工单。
            输出明确的下一步行动方案。
        ")
        .tool(ExperimentalDataReader)
        .temperature(0.3)
        .build();

    println!("✓ 所有 Agent 已就绪\n");

    // ============= 手动编排流程（支持流式输出） =============

    let user_request = "
        我需要开发一种用于高速切削刀具的 TiAlN 涂层。
        
        当前方案：
        - 成分: Al 50%, Ti 40%, N 10%
        - 工艺: 气压0.6 Pa (N2:210 sccm, Ar:280 sccm, Kr:200 sccm)
                偏压90 V, 温度550°C
        - 结构: 单层，厚度 3 μm
        
        目标性能：
        - 硬度 ≥ 3500 HV
        - 附着力 ≥ 70 N
        - 耐磨性优异
        - 可在800°C下稳定工作
        
        请帮我进行性能预测并给出优化建议。
    ";

    println!("{}\n", "=".repeat(60));
    println!("=== 用户需求 ===\n");
    println!("{}", user_request);
    println!("{}\n", "=".repeat(60));

    // 【阶段一：需求提取】
    println!("\n{}\n", "=".repeat(60));
    println!("=== 阶段一：需求提取 ===\n");
    let requirement_prompt = format!(
        "请根据以下信息提取和整理涂层需求参数：\n\n{}",
        user_request
    );
    let _requirement_result = stream_agent_response(&requirement_agent, &requirement_prompt, "需求提取专家").await?;

    // 【阶段二：性能预测】
    println!("\n{}\n", "=".repeat(60));
    println!("=== 阶段二：性能预测 ===\n");
    let prediction_prompt = format!(
        "请基于以下参数进行多维度性能预测：\n\n{}",
        user_request
    );
    let _prediction_result = stream_agent_response(&prediction_agent, &prediction_prompt, "性能预测专家").await?;

    // 【阶段三：优化建议】- 三个优化并行执行
    println!("\n{}\n", "=".repeat(60));
    println!("=== 阶段三：优化建议（并行执行） ===\n");
    
    // 准备三个优化的提示词
    let composition_prompt = format!(
        "基于当前方案（Al 50%, Ti 40%, N 10%）和目标性能（硬度≥3500 HV, 附着力≥70 N, 800°C稳定性），\n\
        请作为成分优化专家提出优化建议：\n\
        1. 分析当前Al/(Al+Ti)比例的局限性\n\
        2. 建议调整Al和Ti的比例以提升硬度与抗氧化性\n\
        3. 考虑高Al含量对残余应力和附着力的潜在负面影响\n\
        4. 提出具体的成分调整方案（如Al 60-65%, Ti 35-40%）\n\
        5. 预测调整后的性能变化趋势\n\
        6. 给出调整依据和协同效应说明。"
    );
    
    let structure_prompt = format!(
        "当前为单层3μm TiAlN涂层，目标用于高速切削刀具，要求高硬度（≥3500 HV）、\n\
        高附着力（≥70 N）和800°C热稳定性。\n\
        请作为结构优化专家（P2）提出优化方案：\n\
        1. 分析单层结构的局限性（如应力集中、界面结合弱等）\n\
        2. 设计多层或梯度结构以提升综合性能\n\
        3. 建议底层、中间层与面层的功能定位\n\
        4. 给出各层厚度分配与总厚度控制策略\n\
        5. 输出具体结构设计方案（如双层、纳米多层或梯度结构）及预期效果。"
    );
    
    let process_prompt = format!(
        "当前工艺参数：气压0.6 Pa (N2:210 sccm, Ar:280 sccm, Kr:200 sccm)，偏压90 V，温度550°C。\n\
        请作为工艺优化专家（P3）提出优化方案：\n\
        1. 分析当前工艺参数的优缺点\n\
        2. 优化气体流量比例\n\
        3. 调整偏压和温度参数\n\
        4. 预测工艺参数调整对性能的影响\n\
        5. 输出具体的工艺优化方案。"
    );
    
    // 并行执行三个优化任务
    let (composition_result, structure_result, process_result) = tokio::try_join!(
        async {
            println!("\n--- P1: 成分优化 ---\n");
            stream_agent_response(&composition_optimizer, &composition_prompt, "成分优化专家").await
        },
        async {
            println!("\n--- P2: 结构优化 ---\n");
            stream_agent_response(&structure_optimizer, &structure_prompt, "结构优化专家").await
        },
        async {
            println!("\n--- P3: 工艺优化 ---\n");
            stream_agent_response(&process_optimizer, &process_prompt, "工艺优化专家").await
        }
    )?;
    
    // 输出结果摘要
    println!("\n{}\n", "=".repeat(60));
    println!("=== 优化建议汇总 ===\n");
    println!("✓ P1 成分优化完成 ({} 字符)", composition_result.len());
    println!("✓ P2 结构优化完成 ({} 字符)", structure_result.len());
    println!("✓ P3 工艺优化完成 ({} 字符)", process_result.len());

    // 【阶段四：迭代优化】
    println!("\n{}\n", "=".repeat(60));
    println!("=== 阶段四：迭代优化 ===\n");
    let iteration_prompt = "
        实验室已完成样品制备（样品编号: TiAlN-OPT-001）。
        请读取实验数据，对比预测结果，并给出下一步优化建议。
    ";
    let _iteration_result = stream_agent_response(&iteration_agent, iteration_prompt, "迭代优化管理专家").await?;

    println!("\n{}\n", "=".repeat(60));
    println!("✓ 系统运行完成\n");
    println!("本示例展示了：");
    println!("  1. 手动编排多 Agent 流程（每个 Agent 独立调用）");
    println!("  2. 每个 Agent 的响应都支持流式输出");
    println!("  3. 清晰的阶段性输出");
    println!("  4. 完整的工业应用工作流\n");

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    // 初始化日志（设置为 ERROR 级别，不显示 INFO 及以下日志）
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::ERROR)
        .with_target(false)
        .init();

    // 运行系统
    create_coating_optimization_system_with_streaming().await?;

    Ok(())
}

