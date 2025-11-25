//! 涂层性能预测及优化专家系统（支持流式输出的手动编排版本）
//! 
//! 这个版本使用手动编排方式，而不是 agent-as-tool 模式，
//! 这样可以确保每个子 agent 的响应都能流式输出，提供更好的用户体验。

use rig::prelude::*;
use rig::agent::{Agent, AgentBuilder};
use rig::completion::{CompletionModel, PromptError};
use rig::streaming::{StreamingPrompt, StreamingChat};
use rig::message::Message;
use std::io::Write;

// ============= 错误类型定义 =============

// ============= 辅助函数：流式调用 agent 并显示输出 =============

/// 流式调用 agent 并实时显示输出，返回完整的历史消息（包括工具调用、工具结果、思考）
async fn stream_agent_response<M: CompletionModel + 'static>(
    agent: &Agent<M>,
    prompt: &str,
    agent_name: &str,
    chat_history: Vec<Message>,
) -> Result<Vec<Message>, PromptError>
where
    <M as CompletionModel>::StreamingResponse: Send,
{
    use futures::StreamExt;
    use rig::streaming::StreamedAssistantContent;
    use rig::agent::MultiTurnStreamItem;
    
    println!("\n【{}】开始处理...", agent_name);
    println!("{}\n", "-".repeat(60));
    print!("Response: ");
    
    // 使用 stream_chat 支持 chat_history，或者使用 stream_prompt().with_history()
    let mut stream = if chat_history.is_empty() {
        agent.stream_prompt(prompt).multi_turn(10).await
    } else {
        agent.stream_chat(prompt, chat_history).multi_turn(10).await
    };
    
    // 手动处理流，收集所有消息（包括工具调用、工具结果、思考、文本）
    // 注意：在多轮对话中，每个轮次可能有多个工具调用-工具结果对
    // 我们需要按照正确的顺序组织消息：工具调用 -> 工具结果 -> 文本响应
    let mut collected_messages = Vec::new();
    let mut last_text = String::new();
    let mut current_tool_calls = Vec::new();
    // 存储工具调用 ID 到 call_id 的映射（用于匹配工具结果）
    let mut tool_call_map: std::collections::HashMap<String, Option<String>> = std::collections::HashMap::new();
    let mut tool_results = Vec::new();
    
    while let Some(chunk_result) = stream.next().await {
        match chunk_result {
            Ok(MultiTurnStreamItem::StreamItem(content)) => {
                match content {
                    StreamedAssistantContent::Text(text) => {
                        print!("{}", text.text);
                        std::io::stdout().flush().unwrap();
                        last_text.push_str(&text.text);
                    }
                    StreamedAssistantContent::ToolCall(tool_call) => {
                        println!("\n\n[🔧 工具调用] {}: {}",
                            tool_call.function.name,
                            tool_call.function.arguments);
                        std::io::stdout().flush().unwrap();
                        // 保存工具调用的 id 和 call_id 的映射（用于后续匹配工具结果）
                        tool_call_map.insert(tool_call.id.clone(), tool_call.call_id.clone());
                        current_tool_calls.push(rig::message::AssistantContent::ToolCall(tool_call));
                    }
                    StreamedAssistantContent::ToolResult { id, result } => {
                        println!("\n\n[✓ 工具结果] {}: {}", id, result);
                        print!("Response: ");
                        std::io::stdout().flush().unwrap();
                        // 从映射中获取对应的 call_id（工具结果的 id 就是工具调用的 id）
                        let call_id = tool_call_map.get(&id).and_then(|x| x.clone());
                        tool_results.push((id, call_id, result));
                    }
                    StreamedAssistantContent::Reasoning(reasoning) => {
                        let reasoning_text = reasoning.reasoning.join("\n");
                        print!("{}", reasoning_text);
                        std::io::stdout().flush().unwrap();
                        // 思考过程也作为 AssistantContent::Reasoning 保存
                        collected_messages.push(Message::Assistant {
                            id: None,
                            content: rig::OneOrMany::one(rig::message::AssistantContent::Reasoning(reasoning)),
                        });
                    }
                    StreamedAssistantContent::Final(_) => {
                        // Final 在 MultiTurnStreamItem::FinalResponse 中处理
                    }
                    StreamedAssistantContent::ToolCallDelta { .. } => {
                        // 工具调用增量更新，不需要特殊处理
                    }
                }
            }
            Ok(MultiTurnStreamItem::FinalResponse(final_response)) => {
                // 最终响应中的文本（如果流式没有文本，这里会有）
                let final_text = final_response.response();
                if !final_text.is_empty() && last_text.is_empty() {
                    println!("{}", final_text);
                    last_text.push_str(final_text);
                }
            }
            Ok(_) => {
                // 处理其他可能的 MultiTurnStreamItem 变体（non_exhaustive）
                // 目前框架中只有 StreamItem 和 FinalResponse，但为了兼容性保留此分支
            }
            Err(e) => {
                eprintln!("\n❌ 错误: {}", e);
                return Err(PromptError::CompletionError(
                    rig::completion::CompletionError::ResponseError(e.to_string())
                ));
            }
        }
    }
    
    // 构建完整的消息列表，按照正确的顺序：
    // 1. 如果有工具调用，先添加工具调用消息（Assistant 消息）
    if !current_tool_calls.is_empty() {
        collected_messages.push(Message::Assistant {
            id: None,
            content: rig::OneOrMany::many(current_tool_calls)
                .expect("工具调用列表不应为空"),
        });
    }
    
    // 2. 如果有工具结果，添加工具结果消息（User 消息，包含工具调用的返回结果）
    //    工具结果应该紧跟在对应的工具调用之后
    for (id, call_id, result) in tool_results {
        let tool_result_msg = if let Some(call_id) = call_id {
            // 使用 tool_result_with_call_id 来正确关联工具调用和结果
            Message::User {
                content: rig::OneOrMany::one(rig::message::UserContent::tool_result_with_call_id(
                    id,
                    call_id,
                    rig::OneOrMany::one(rig::message::ToolResultContent::Text(
                        rig::message::Text { text: result }
                    )),
                )),
            }
        } else {
            // 如果没有 call_id，使用普通的 tool_result
            Message::User {
                content: rig::OneOrMany::one(rig::message::UserContent::ToolResult(
                    rig::message::ToolResult {
                        id,
                        call_id: None,
                        content: rig::OneOrMany::one(rig::message::ToolResultContent::Text(
                            rig::message::Text { text: result }
                        )),
                    }
                )),
            }
        };
        collected_messages.push(tool_result_msg);
    }
    
    // 3. 如果有文本响应，添加文本消息（这是最终的回答）
    //    注意：如果只有工具调用和工具结果，没有文本响应，这是正常的（多轮对话中）
    if !last_text.is_empty() {
        collected_messages.push(Message::Assistant {
            id: None,
            content: rig::OneOrMany::one(rig::message::AssistantContent::Text(
                rig::message::Text { text: last_text }
            )),
        });
    }
    
    println!("\n{}\n", "-".repeat(60));
    println!("【{}】完成\n", agent_name);
    
    Ok(collected_messages)
}

// ============= 工作流上下文 =============

/// 工作流上下文，使用 chat_history 累积每个阶段的处理结果
struct WorkflowContext {
    chat_history: Vec<Message>,
}

impl WorkflowContext {
    fn new(original_request: String) -> Self {
        Self {
            chat_history: vec![Message::user(original_request)],
        }
    }

    /// 获取当前的 chat_history
    fn get_history(&self) -> Vec<Message> {
        self.chat_history.clone()
    }

    /// 添加用户消息到 chat_history
    fn add_user_message(&mut self, message: String) {
        self.chat_history.push(Message::user(message));
    }

    /// 添加助手回复到 chat_history（用于累积上下文）
    fn add_assistant_message(&mut self, message: String) {
        self.chat_history.push(Message::assistant(message));
    }

    /// 添加完整的消息列表到 chat_history（包括工具调用、工具结果、思考等）
    fn add_messages(&mut self, messages: Vec<Message>) {
        self.chat_history.extend(messages);
    }

    /// 获取当前上下文摘要（用于显示）
    fn get_summary(&self) -> String {
        format!("聊天历史包含 {} 条消息", self.chat_history.len())
    }
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
        .tool(rig::tools::TopPhiSimulator)
        .tool(rig::tools::MLPerformancePredictor)
        .tool(rig::tools::HistoricalDataQuery)
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
        .tool(rig::tools::ExperimentalDataReader)
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

    // 初始化工作流上下文（使用 chat_history）
    let mut ctx = WorkflowContext::new(user_request.to_string());

    // 【阶段一：需求提取】
    println!("\n{}\n", "=".repeat(60));
    println!("=== 阶段一：需求提取 ===\n");
    let requirement_prompt = "请根据聊天历史中的信息提取和整理涂层需求参数。";
    let requirement_messages = stream_agent_response(
        &requirement_agent, 
        requirement_prompt,
        "需求提取专家",
        ctx.get_history()
    ).await?;
    ctx.add_messages(requirement_messages);
    println!("✓ 需求提取结果（包括工具调用和工具结果）已添加到 chat_history");

    // 【阶段二：性能预测】（使用 chat_history，包含阶段一的结果）
    println!("\n{}\n", "=".repeat(60));
    println!("=== 阶段二：性能预测（基于 chat_history，包含需求提取结果） ===\n");
    let prediction_prompt = "请基于聊天历史中的信息进行多维度性能预测。";
    let prediction_messages = stream_agent_response(
        &prediction_agent,
        prediction_prompt,
        "性能预测专家",
        ctx.get_history()
    ).await?;
    ctx.add_messages(prediction_messages);
    println!("✓ 性能预测结果（包括工具调用和工具结果）已添加到 chat_history");

    // 【阶段三：优化建议】（使用 chat_history，包含阶段一和阶段二的结果）
    println!("\n{}\n", "=".repeat(60));
    println!("=== 阶段三：优化建议（基于 chat_history） ===\n");
    
    // P1: 成分优化
    println!("\n--- P1: 成分优化 ---\n");
    let composition_prompt = "请作为成分优化专家，基于聊天历史中的信息提出优化建议：\n\
        1. 分析当前Al/(Al+Ti)比例的局限性\n\
        2. 建议调整Al和Ti的比例以提升硬度与抗氧化性\n\
        3. 考虑高Al含量对残余应力和附着力的潜在负面影响\n\
        4. 提出具体的成分调整方案（如Al 60-65%, Ti 35-40%）\n\
        5. 预测调整后的性能变化趋势\n\
        6. 给出调整依据和协同效应说明。";
    let composition_messages = stream_agent_response(
        &composition_optimizer, 
        composition_prompt, 
        "成分优化专家",
        ctx.get_history()
    ).await?;
    ctx.add_messages(composition_messages);
    println!("✓ 成分优化结果已添加到 chat_history");

    // P2: 结构优化
    println!("\n--- P2: 结构优化 ---\n");
    let structure_prompt = "请作为结构优化专家（P2），基于聊天历史中的信息提出优化方案：\n\
        1. 分析单层结构的局限性（如应力集中、界面结合弱等）\n\
        2. 设计多层或梯度结构以提升综合性能\n\
        3. 建议底层、中间层与面层的功能定位\n\
        4. 给出各层厚度分配与总厚度控制策略\n\
        5. 输出具体结构设计方案（如双层、纳米多层或梯度结构）及预期效果。";
    let structure_messages = stream_agent_response(
        &structure_optimizer, 
        structure_prompt, 
        "结构优化专家",
        ctx.get_history()
    ).await?;
    ctx.add_messages(structure_messages);
    println!("✓ 结构优化结果已添加到 chat_history");

    // P3: 工艺优化
    println!("\n--- P3: 工艺优化 ---\n");
    let process_prompt = "请作为工艺优化专家（P3），基于聊天历史中的信息提出优化方案：\n\
        1. 分析当前工艺参数的优缺点\n\
        2. 优化气体流量比例\n\
        3. 调整偏压和温度参数\n\
        4. 预测工艺参数调整对性能的影响\n\
        5. 输出具体的工艺优化方案。";
    let process_messages = stream_agent_response(
        &process_optimizer, 
        process_prompt, 
        "工艺优化专家",
        ctx.get_history()
    ).await?;
    ctx.add_messages(process_messages);
    println!("✓ 工艺优化结果已添加到 chat_history");

    // 【阶段四：迭代优化】（使用 chat_history，包含所有前面的结果）
    println!("\n{}\n", "=".repeat(60));
    println!("=== 阶段四：迭代优化（基于 chat_history，包含所有前面阶段的结果） ===\n");
    let iteration_prompt = "实验室已完成样品制备（样品编号: TiAlN-OPT-001）。\n\
        请读取实验数据，对比聊天历史中的预测结果，并给出下一步优化建议。";
    let iteration_messages = stream_agent_response(
        &iteration_agent, 
        iteration_prompt, 
        "迭代优化管理专家",
        ctx.get_history()
    ).await?;
    ctx.add_messages(iteration_messages);
    println!("✓ 迭代优化结果（包括工具调用和工具结果）已添加到 chat_history");

    // println!("chat_history: {:?}", ctx.get_history());
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

