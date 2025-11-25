//! Agent 编排示例
//! 
//! 演示多种 Agent 编排模式：
//! 1. Agent 作为工具
//! 2. 编排器模式
//! 3. 专业 Agent 协作

use rig::prelude::*;
use rig::agent::{Agent, AgentBuilder};
use rig::completion::{Chat, CompletionModel, Prompt, PromptError, ToolDefinition};
use rig::providers::openai::Client;
use rig::tool::Tool;
use serde::Deserialize;
use serde_json::json;

// ============= Agent 工具包装器 =============

// 包装翻译 Agent 作为工具
struct TranslatorTool<M: CompletionModel>(Agent<M>);

#[derive(Deserialize)]
struct TranslatorArgs {
    text: String,
}

impl<M: CompletionModel> Tool for TranslatorTool<M> {
    const NAME: &'static str = "translator";
    type Args = TranslatorArgs;
    type Error = PromptError;
    type Output = String;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        serde_json::from_value(json!({
            "name": "translator",
            "description": "将文本翻译成英文，或修正英文语法错误",
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
        }))
        .expect("Tool Definition")
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        println!("[翻译工具] 翻译文本: {}", args.text);
        let result = self.0.chat(&args.text, vec![]).await?;
        println!("[翻译工具] 翻译结果: {}", result);
        Ok(result)
    }
}

// 包装研究 Agent 作为工具
struct ResearchTool<M: CompletionModel>(Agent<M>);

#[derive(Deserialize)]
struct ResearchArgs {
    topic: String,
}

impl<M: CompletionModel> Tool for ResearchTool<M> {
    const NAME: &'static str = "researcher";
    type Args = ResearchArgs;
    type Error = PromptError;
    type Output = String;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        serde_json::from_value(json!({
            "name": "researcher",
            "description": "对特定主题进行深入研究，提供详细的技术信息",
            "parameters": {
                "type": "object",
                "properties": {
                    "topic": {
                        "type": "string",
                        "description": "要研究的主题"
                    }
                },
                "required": ["topic"]
            }
        }))
        .expect("Tool Definition")
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        println!("[研究工具] 研究主题: {}", args.topic);
        let result = self.0.chat(&args.topic, vec![]).await?;
        Ok(result)
    }
}

// 包装分析 Agent 作为工具
struct AnalysisTool<M: CompletionModel>(Agent<M>);

#[derive(Deserialize)]
struct AnalysisArgs {
    data: String,
}

impl<M: CompletionModel> Tool for AnalysisTool<M> {
    const NAME: &'static str = "analyzer";
    type Args = AnalysisArgs;
    type Error = PromptError;
    type Output = String;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        serde_json::from_value(json!({
            "name": "analyzer",
            "description": "分析数据和信息，提供洞察和结论",
            "parameters": {
                "type": "object",
                "properties": {
                    "data": {
                        "type": "string",
                        "description": "要分析的数据或信息"
                    }
                },
                "required": ["data"]
            }
        }))
        .expect("Tool Definition")
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        println!("[分析工具] 分析数据");
        let result = self.0.chat(&args.data, vec![]).await?;
        Ok(result)
    }
}

// 包装总结 Agent 作为工具
struct SummarizerTool<M: CompletionModel>(Agent<M>);

#[derive(Deserialize)]
struct SummarizerArgs {
    content: String,
}

impl<M: CompletionModel> Tool for SummarizerTool<M> {
    const NAME: &'static str = "summarizer";
    type Args = SummarizerArgs;
    type Error = PromptError;
    type Output = String;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        serde_json::from_value(json!({
            "name": "summarizer",
            "description": "总结长文本，提取关键信息",
            "parameters": {
                "type": "object",
                "properties": {
                    "content": {
                        "type": "string",
                        "description": "要总结的内容"
                    }
                },
                "required": ["content"]
            }
        }))
        .expect("Tool Definition")
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        println!("[总结工具] 总结内容（长度: {} 字符）", args.content.len());
        let result = self.0.chat(&args.content, vec![]).await?;
        Ok(result)
    }
}

// ============= 示例函数 =============

async fn example_agent_as_tool(client: &Client) -> Result<(), anyhow::Error> {
    println!("=== 示例 1: Agent 作为工具 ===\n");

    // 创建翻译 Agent
    let translator_agent = client
        .agent("gpt-4o")
        .name("翻译专家")
        .preamble(
            "你是一位专业的翻译。\
            将任何非英文文本翻译成英文。\
            如果已经是英文，则修正语法和拼写错误。\
            只返回翻译后的文本，不要添加额外说明。"
        )
        .temperature(0.3)
        .build();

    // 将翻译 Agent 包装成工具
    let translator_tool = TranslatorTool(translator_agent);

    // 创建使用翻译工具的主 Agent
    let main_agent = client
        .agent("gpt-4o")
        .name("多语言助理")
        .preamble(&format!(
            "你是一个多语言助理。\
            当收到非英文输入时，先使用 {} 工具翻译成英文。\
            然后用英文回答用户的问题。\
            最后将答案翻译回原始语言。",
            TranslatorTool::<_>::NAME
        ))
        .tool(translator_tool)
        .build();

    let response = main_agent
        .prompt("你好，请介绍一下 Rust 编程语言的优势。")
        .multi_turn(5)
        .await?;

    println!("主 Agent 回复:\n{}\n", response);

    Ok(())
}

async fn example_orchestrator_pattern(client: &Client) -> Result<(), anyhow::Error> {
    println!("=== 示例 2: 编排器模式 ===\n");

    let model = client.completion_model("gpt-4o");

    // 创建专业 Agent
    let research_agent = AgentBuilder::new(model.clone())
        .name("研究专员")
        .preamble(
            "你是技术研究专家。\
            对用户提供的主题进行深入研究，提供详细、准确的技术信息。\
            你的回答应该包含具体的技术细节、优缺点和实际应用场景。"
        )
        .temperature(0.3)
        .build();

    let analysis_agent = AgentBuilder::new(model.clone())
        .name("数据分析师")
        .preamble(
            "你是数据分析专家。\
            分析提供的信息，识别模式、趋势和关键洞察。\
            提供清晰的分析结论和建议。"
        )
        .temperature(0.3)
        .build();

    let summarizer_agent = AgentBuilder::new(model.clone())
        .name("总结专家")
        .preamble(
            "你是信息总结专家。\
            将复杂的信息总结成简洁、易懂的要点。\
            保留关键信息，去除冗余内容。"
        )
        .temperature(0.3)
        .build();

    // 将 Agent 包装成工具
    let research_tool = ResearchTool(research_agent);
    let analysis_tool = AnalysisTool(analysis_agent);
    let summarizer_tool = SummarizerTool(summarizer_agent);

    // 创建编排器 Agent
    let orchestrator = AgentBuilder::new(model)
        .name("主编排器")
        .preamble(&format!(
            "你是项目协调员，负责协调专业团队完成任务。\n\n\
            可用的专业团队：\n\
            1. {} - 进行深入技术研究\n\
            2. {} - 分析数据和信息\n\
            3. {} - 总结和精炼内容\n\n\
            工作流程：\n\
            1. 使用 researcher 收集详细的技术信息\n\
            2. 使用 analyzer 分析研究结果\n\
            3. 使用 summarizer 生成简洁的最终报告\n\n\
            按照这个流程协调团队，为用户提供高质量的输出。",
            ResearchTool::<_>::NAME,
            AnalysisTool::<_>::NAME,
            SummarizerTool::<_>::NAME
        ))
        .tool(research_tool)
        .tool(analysis_tool)
        .tool(summarizer_tool)
        .temperature(0.5)
        .build();

    let response = orchestrator
        .prompt("分析 Rust 编程语言在 WebAssembly 领域的应用前景")
        .multi_turn(15)
        .await?;

    println!("编排器最终报告:\n{}\n", response);

    Ok(())
}

async fn example_specialized_team(client: &Client) -> Result<(), anyhow::Error> {
    println!("=== 示例 3: 专业团队协作 ===\n");

    let model = client.completion_model("gpt-4o");

    // 创建客户服务团队
    let product_expert = AgentBuilder::new(model.clone())
        .name("产品专家")
        .preamble("你是产品专家，了解所有产品的详细规格和特性。")
        .context("产品 A: 智能手表 - 价格 ¥1999，功能：健康监测、消息提醒、支付")
        .context("产品 B: 无线耳机 - 价格 ¥599，功能：主动降噪、30小时续航")
        .context("产品 C: 智能音箱 - 价格 ¥299，功能：语音助手、智能家居控制")
        .build();

    let order_expert = AgentBuilder::new(model.clone())
        .name("订单专家")
        .preamble("你是订单处理专家，可以查询订单状态和物流信息。")
        .context("订单状态：pending（待处理）、shipped（已发货）、delivered（已送达）")
        .build();

    let refund_expert = AgentBuilder::new(model.clone())
        .name("退款专家")
        .preamble("你是退款处理专家，处理退款请求。")
        .context("退款政策：购买后 30 天内可无理由退货")
        .context("退款流程：提交申请 -> 审核 -> 退货 -> 退款")
        .build();

    // 创建客服主管 Agent
    let customer_service = client
        .agent("gpt-4o")
        .name("客服主管")
        .preamble(
            "你是客户服务主管，负责协调专业团队处理客户咨询。\n\n\
            根据客户问题类型，委派给适当的专家：\n\
            - 产品咨询 -> 产品专家\n\
            - 订单查询 -> 订单专家\n\
            - 退款退货 -> 退款专家\n\n\
            整合专家的回答，为客户提供完整、专业的服务。\
            始终保持礼貌和专业。"
        )
        .tool(product_expert)
        .tool(order_expert)
        .tool(refund_expert)
        .build();

    // 测试多个客户咨询
    let queries = vec![
        "我想了解智能手表的功能和价格",
        "订单 #12345 的状态是什么？",
        "我想退货，流程是什么？",
        "我想买无线耳机，但不确定是否支持我的手机",
    ];

    for (i, query) in queries.iter().enumerate() {
        println!("--- 客户咨询 {} ---", i + 1);
        println!("客户: {}", query);

        let response = customer_service
            .prompt(query)
            .multi_turn(5)
            .await?;

        println!("客服: {}\n", response);
    }

    Ok(())
}

// ============= 主程序 =============

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .init();

    let client = Client::from_env();

    // 运行示例
    example_agent_as_tool(&client).await?;
    println!("\n{'='*60}\n");

    example_orchestrator_pattern(&client).await?;
    println!("\n{'='*60}\n");

    example_specialized_team(&client).await?;

    println!("所有示例完成！");

    Ok(())
}

