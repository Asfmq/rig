//! 涂层性能预测及优化专家系统
//! 
//! 这是一个完整的多 Agent 编排示例，展示如何使用 qwen-plus 构建复杂的工业应用系统
//! 
//! 系统架构：
//! 1. 需求提取 Agent - 收集和验证输入参数
//! 2. 性能预测 Agent - 预测涂层性能
//! 3. 优化建议 Agent - 提供分类优化方案
//! 4. 迭代优化 Agent - 管理优化迭代流程
//! 5. 主编排 Agent - 协调整个流程

use rig::prelude::*;
use rig::agent::{Agent, AgentBuilder, stream_to_stdout};
use rig::completion::{Chat, CompletionModel, Prompt, PromptError, ToolDefinition};
use rig::streaming::StreamingPrompt;
use rig::tool::Tool;
use serde::{Deserialize, Serialize};
use serde_json::json;

// ============= 错误类型定义 =============

#[derive(Debug, thiserror::Error)]
enum ToolError {
    #[error("模拟工具错误: {0}")]
    SimulationError(String),
}

// ============= 数据结构定义 =============

/// 涂层成分
#[derive(Debug, Clone, Serialize, Deserialize)]
struct CoatingComposition {
    al: f64,  // 铝含量
    ti: f64,  // 钛含量
    n: f64,   // 氮含量
    x: f64,   // 其他元素X含量
}

/// 工艺参数
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ProcessParameters {
    n2_flow: f64,      // N2流量 (sccm)
    ar_flow: f64,      // Ar流量 (sccm)
    kr_flow: f64,      // Kr流量 (sccm)
    pressure: f64,     // 沉积气压 (Pa)
    bias_voltage: f64, // 偏压 (V)
    temperature: f64,  // 沉积温度 (°C)
}

/// 涂层结构
#[derive(Debug, Clone, Serialize, Deserialize)]
struct CoatingStructure {
    total_thickness: f64,        // 总厚度 (μm)
    layer_ratios: Vec<f64>,      // 各层占比
    layer_descriptions: Vec<String>, // 各层描述
}

/// 性能需求
#[derive(Debug, Clone, Serialize, Deserialize)]
struct PerformanceRequirements {
    application_scenario: String, // 应用场景
    target_hardness: f64,         // 目标硬度 (HV)
    target_adhesion: f64,         // 目标附着力
    wear_resistance: String,      // 耐磨性要求
    other_requirements: Vec<String>, // 其他需求
}

/// 预测结果
#[derive(Debug, Clone, Serialize, Deserialize)]
struct PredictionResult {
    predicted_hardness: f64,
    predicted_adhesion: f64,
    structure_morphology: String,
    confidence_score: f64,
    historical_comparison: String,
}

/// 优化建议
#[derive(Debug, Clone, Serialize, Deserialize)]
struct OptimizationSuggestion {
    category: String, // P1: 成分, P2: 结构, P3: 工艺
    suggestions: Vec<String>,
    expected_improvement: f64,
    priority: i32,
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
                    "composition": {
                        "type": "string",
                        "description": "涂层成分信息（JSON格式）"
                    },
                    "process_params": {
                        "type": "string",
                        "description": "工艺参数（JSON格式）"
                    },
                    "structure": {
                        "type": "string",
                        "description": "预计沉积结构（JSON格式）"
                    }
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
        
        // 模拟输出
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
                    "composition": {
                        "type": "string",
                        "description": "涂层成分"
                    },
                    "process_params": {
                        "type": "string",
                        "description": "工艺参数"
                    },
                    "structure": {
                        "type": "string",
                        "description": "涂层结构"
                    },
                    "simulation_result": {
                        "type": "string",
                        "description": "TopPhi模拟结果"
                    }
                },
                "required": ["composition", "process_params", "structure", "simulation_result"]
            }
        }))
        .expect("Tool Definition")
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        println!("\n[ML性能预测器] 使用机器学习模型预测性能...");
        
        // 模拟 ML 预测
        let result = json!({
            "predicted_hardness": 3250.0,
            "hardness_confidence": 0.92,
            "predicted_adhesion": 68.5,
            "adhesion_confidence": 0.88,
            "predicted_wear_rate": 1.2e-6,
            "wear_confidence": 0.85,
            "friction_coefficient": 0.35,
            "model_version": "v2.3.1",
            "feature_importance": {
                "composition": 0.45,
                "process_temp": 0.25,
                "bias_voltage": 0.18,
                "structure": 0.12
            }
        });
        
        println!("  ✓ 预测完成");
        println!("    - 硬度: 3250 HV (置信度: 92%)");
        println!("    - 附着力: 68.5 N (置信度: 88%)");
        println!("    - 磨损率: 1.2×10⁻⁶ mm³/Nm (置信度: 85%)\n");
        
        Ok(serde_json::to_string_pretty(&result)
            .map_err(|e| ToolError::SimulationError(e.to_string()))?)
    }
}

/// 历史数据查询工具（模拟）
#[derive(Deserialize, Serialize)]
struct HistoricalDataQuery;

#[derive(Deserialize)]
struct HistoricalQueryArgs {
    composition_range: String,
    process_range: String,
}

impl Tool for HistoricalDataQuery {
    const NAME: &'static str = "historical_data_query";
    type Error = ToolError;
    type Args = HistoricalQueryArgs;
    type Output = String;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        serde_json::from_value(json!({
            "name": "historical_data_query",
            "description": "查询历史实验数据库，找到相似配方的历史记录",
            "parameters": {
                "type": "object",
                "properties": {
                    "composition_range": {
                        "type": "string",
                        "description": "成分范围"
                    },
                    "process_range": {
                        "type": "string",
                        "description": "工艺参数范围"
                    }
                },
                "required": ["composition_range", "process_range"]
            }
        }))
        .expect("Tool Definition")
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        println!("\n[历史数据库] 查询相似配方...");
        println!("  - 成分范围: {}", args.composition_range);
        println!("  - 工艺范围: {}", args.process_range);
        
        // 模拟查询结果
        let result = json!({
            "matched_records": 5,
            "similar_samples": [
                {
                    "sample_id": "TiAlN-2023-045",
                    "composition": {"Al": 0.52, "Ti": 0.38, "N": 0.10},
                    "hardness": 3180.0,
                    "adhesion": 65.2,
                    "similarity": 0.94
                },
                {
                    "sample_id": "TiAlN-2023-072",
                    "composition": {"Al": 0.50, "Ti": 0.40, "N": 0.10},
                    "hardness": 3320.0,
                    "adhesion": 70.1,
                    "similarity": 0.91
                },
                {
                    "sample_id": "TiAlN-2022-156",
                    "composition": {"Al": 0.48, "Ti": 0.42, "N": 0.10},
                    "hardness": 3050.0,
                    "adhesion": 62.8,
                    "similarity": 0.87
                }
            ],
            "performance_range": {
                "hardness": {"min": 3050.0, "max": 3320.0, "avg": 3183.3},
                "adhesion": {"min": 62.8, "max": 70.1, "avg": 66.0}
            }
        });
        
        println!("  ✓ 找到 5 条相似记录");
        println!("    - 硬度范围: 3050-3320 HV");
        println!("    - 附着力范围: 62.8-70.1 N\n");
        
        Ok(serde_json::to_string_pretty(&result)
            .map_err(|e| ToolError::SimulationError(e.to_string()))?)
    }
}

/// 实验数据读取工具（模拟）
#[derive(Deserialize, Serialize)]
struct ExperimentalDataReader;

#[derive(Deserialize)]
struct ExperimentDataArgs {
    sample_id: String,
}

impl Tool for ExperimentalDataReader {
    const NAME: &'static str = "experimental_data_reader";
    type Error = ToolError;
    type Args = ExperimentDataArgs;
    type Output = String;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        serde_json::from_value(json!({
            "name": "experimental_data_reader",
            "description": "读取实验验证后的实际性能数据（硬度、SEM图像分析等）",
            "parameters": {
                "type": "object",
                "properties": {
                    "sample_id": {
                        "type": "string",
                        "description": "样品编号"
                    }
                },
                "required": ["sample_id"]
            }
        }))
        .expect("Tool Definition")
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        println!("\n[实验数据读取] 读取样品 {} 的实验数据...", args.sample_id);
        
        // 模拟实验数据
        let result = json!({
            "sample_id": args.sample_id,
            "measured_hardness": 3285.0,
            "hardness_std_dev": 45.0,
            "measured_adhesion": 67.2,
            "adhesion_std_dev": 2.3,
            "sem_analysis": {
                "grain_size": "55-75 nm",
                "phase_composition": "主相: 面心立方 TiAlN, 次相: 六方 AlN (约5%)",
                "surface_quality": "均匀致密，无裂纹",
                "interface_bonding": "良好"
            },
            "xrd_peaks": ["TiAlN(111)", "TiAlN(200)", "TiAlN(220)", "AlN(002)"],
            "test_date": "2025-10-31",
            "operator": "实验室-02"
        });
        
        println!("  ✓ 数据读取完成");
        println!("    - 实测硬度: 3285 ± 45 HV");
        println!("    - 实测附着力: 67.2 ± 2.3 N");
        println!("    - SEM分析: 晶粒尺寸 55-75 nm\n");
        
        Ok(serde_json::to_string_pretty(&result)
            .map_err(|e| ToolError::SimulationError(e.to_string()))?)
    }
}

// ============= Agent 工具包装器 =============

/// 包装 Agent 为工具的通用结构
struct AgentTool<M: CompletionModel> {
    agent: Agent<M>,
    tool_name: &'static str,
    description: String,
}

impl<M: CompletionModel> AgentTool<M> {
    fn new(agent: Agent<M>, tool_name: &'static str, description: String) -> Self {
        Self {
            agent,
            tool_name,
            description,
        }
    }
}

#[derive(Deserialize)]
struct AgentToolArgs {
    input: String,
}

impl<M: CompletionModel> Tool for AgentTool<M> {
    const NAME: &'static str = "agent_tool";
    type Error = PromptError;
    type Args = AgentToolArgs;
    type Output = String;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        serde_json::from_value(json!({
            "name": self.tool_name,
            "description": self.description,
            "parameters": {
                "type": "object",
                "properties": {
                    "input": {
                        "type": "string",
                        "description": "输入信息"
                    }
                },
                "required": ["input"]
            }
        }))
        .expect("Tool Definition")
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        println!("\n[{}] 处理中...", self.tool_name);
        let result = self.agent.chat(&args.input, vec![]).await?;
        println!("  ✓ 完成\n");
        Ok(result)
    }
}

// ============= 创建专业 Agent =============

async fn create_coating_optimization_system(
) -> Result<(), anyhow::Error> {
    
    // 使用 Qwen Plus 模型
    let api_key = "sk-348d7ca647714c52aca12ea106cfa895";
    // let qwen_client = rig::providers::qwen::Client::new_with_api_key(&api_key);
    // let model = qwen_client.completion_model("qwen-plus");
    let qwen_client = rig::providers::ollama::Client::new();
    let model = qwen_client.completion_model("llama3.2");

    println!("=== 涂层性能预测及优化专家系统 ===\n");
    println!("正在初始化 Agent 系统...\n");

    // 1. 需求提取 Agent
    let requirement_agent = AgentBuilder::new(model.clone())
        .name("需求提取专家")
        .preamble("
            你是涂层需求提取专家。负责：
            
            职责：
            1. 收集和整理涂层成分信息（Al、Ti、N、X元素含量）
            2. 记录工艺参数（气压、流量、偏压、温度）
            3. 确认涂层结构信息（厚度、分层）
            4. 明确应用场景和性能需求
            5. 验证数据完整性和合理性
            
            输出格式：
            - 结构化的JSON格式数据
            - 标注缺失或异常的参数
            - 提供参数合理性评估
            
            风格：严谨、细致、专业
        ")
        .temperature(0.3)
        .build();

    // 2. 性能预测 Agent
    let prediction_agent = AgentBuilder::new(model.clone())
        .name("性能预测专家")
        .preamble("
            你是涂层性能预测专家。负责：
            
            职责：
            1. 调用 TopPhi 模拟器预测沉积形貌
            2. 使用 ML 模型预测性能指标
            3. 查询历史数据进行对比
            4. 进行根因分析
            5. 评估预测置信度
            
            分析维度：
            - 微观结构（晶粒尺寸、相组成）
            - 力学性能（硬度、附着力）
            - 摩擦学性能（耐磨性、摩擦系数）
            - 与历史数据的差异分析
            
            风格：科学、客观、数据驱动
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
            你是涂层成分优化专家（P1优化）。
            
            职责：
            1. 分析当前成分配比的优缺点
            2. 基于性能目标提出成分调整建议
            3. 考虑元素间协同效应
            4. 预测成分调整后的性能变化
            
            优化原则：
            - Al含量影响氧化温度和硬度
            - Ti含量影响韧性和结合力
            - N含量影响相组成
            - 元素X的特殊作用
            
            输出：具体的成分调整方案和理由
            
            风格：专业、有理有据、可操作
        ")
        .temperature(0.4)
        .build();

    // 4. 结构优化 Agent
    let structure_optimizer = AgentBuilder::new(model.clone())
        .name("结构优化专家")
        .preamble("
            你是涂层结构优化专家（P2优化）。
            
            职责：
            1. 设计多层结构方案
            2. 优化各层厚度和占比
            3. 设计梯度或纳米多层结构
            4. 考虑应力释放和界面结合
            
            结构类型：
            - 单层结构
            - 双层结构（底层+面层）
            - 多层结构（>3层）
            - 纳米多层（周期<100nm）
            - 梯度结构
            
            输出：详细的结构设计方案
            
            风格：创新、系统化、工程化
        ")
        .temperature(0.5)
        .build();

    // 5. 工艺优化 Agent
    let process_optimizer = AgentBuilder::new(model.clone())
        .name("工艺优化专家")
        .preamble("
            你是涂层工艺优化专家（P3优化）。
            
            职责：
            1. 优化沉积气压和气体流量
            2. 调整偏压和沉积温度
            3. 优化沉积速率
            4. 控制应力和致密度
            
            工艺参数影响：
            - 气压: 影响离子轰击和沉积速率
            - 流量比: 影响相组成和化学计量
            - 偏压: 影响致密度和应力
            - 温度: 影响晶粒生长和相变
            
            输出：参数调整方案和预期效果
            
            风格：精确、经验丰富、实用
        ")
        .temperature(0.4)
        .build();

    // 6. 迭代优化 Agent
    let iteration_agent = AgentBuilder::new(model.clone())
        .name("迭代优化管理专家")
        .preamble("
            你是迭代优化流程管理专家。
            
            职责：
            1. 管理优化迭代流程
            2. 比对预测值与实测值
            3. 分析偏差原因
            4. 决定下一步优化方向
            5. 生成试验工单
            
            决策逻辑：
            - 如果性能达标: 验证稳定性，结束流程
            - 如果接近目标: 微调优化
            - 如果偏差较大: 重新评估优化路径
            - 如果多次失败: 调整优化策略
            
            输出：明确的下一步行动方案
            
            风格：系统化、决策清晰、目标导向
        ")
        .tool(ExperimentalDataReader)
        .temperature(0.3)
        .build();


    // 7. 创建主编排 Agent
    let orchestrator = AgentBuilder::new(model.clone())
        .name("涂层优化系统总控")
        .preamble("
            你是涂层性能预测及优化系统的总协调者。
            
            工作流程：
            
            【阶段一：需求提取】
            1. 使用需求提取专家收集完整参数
            2. 验证数据完整性
            
            【阶段二：性能预测】
            1. 使用性能预测专家进行多维度预测
            2. 获取TopPhi模拟结果
            3. 获取ML模型预测
            4. 进行历史数据比对
            5. 输出综合预测报告
            
            【阶段三：优化建议】
            根据预测结果和性能差距，调用适当的优化专家：
            - P1: 成分优化专家 - 调整元素配比
            - P2: 结构优化专家 - 优化层状结构
            - P3: 工艺优化专家 - 调整工艺参数
            可以组合多种优化方案
            
            【阶段四：迭代优化】
            1. 使用迭代优化管理专家跟踪实验
            2. 对比预测与实测数据
            3. 决定是否需要进一步优化
            4. 生成下一轮优化方案
            
            协调原则：
            - 系统化推进，不跳过步骤
            - 充分利用各专家的专业能力
            - 基于数据做决策
            - 清晰的阶段性输出
            
            风格：专业、系统化、高效
        ")
        .tool(requirement_agent)
        .tool(prediction_agent)
        .tool(composition_optimizer)
        .tool(structure_optimizer)
        .tool(process_optimizer)
        .tool(iteration_agent)
        .temperature(0.5)
        .build();

    println!("✓ 系统总控 - 就绪");
    println!("\n{}\n", "=".repeat(60));

    // ============= 运行完整流程 =============

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

    println!("=== 用户需求 ===");
    println!("{}", user_request);

    // 执行主流程（流式输出）
    println!("\n{}\n", "=".repeat(60));
    println!("=== 系统输出（流式） ===\n");
    
    let mut stream = orchestrator
        .stream_prompt(user_request)
        .multi_turn(25) // 允许多轮交互以完成复杂流程
        .await;
    
    let res = stream_to_stdout(&mut stream).await?;
    
    println!("\n{}\n", "=".repeat(60));
    println!("Token 使用: {:?}", res.usage());
    println!("\n{}\n", "=".repeat(60));

    // 模拟后续的迭代流程
    println!("\n=== 模拟实验验证和迭代 ===\n");
    
    let iteration_request = "
        实验室已完成样品制备（样品编号: TiAlN-OPT-001）。
        请读取实验数据，对比预测结果，并给出下一步优化建议。
    ";

    println!("迭代请求: {}\n", iteration_request);

    println!("\n=== 迭代优化结果（流式） ===\n");
    
    let mut iteration_stream = orchestrator
        .stream_prompt(iteration_request)
        .multi_turn(15)
        .await;
    
    let iteration_res = stream_to_stdout(&mut iteration_stream).await?;
    
    println!("\n\nToken 使用: {:?}", iteration_res.usage());

    Ok(())
}

// ============= 主函数 =============

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    // 初始化日志（设置为 ERROR 级别，不显示 INFO 及以下日志）
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::ERROR)
        .with_target(false)
        .init();

    // 运行系统
    create_coating_optimization_system().await?;


    Ok(())
}

