//! 通义千问（Qwen）流式输出示例（使用 MCP 服务器）
//!
//! 本示例展示如何使用 Rig 框架结合通义千问和外部 MCP（Model Context Protocol）服务器
//! 实现流式响应和工具调用功能
//!
//! 运行示例：
//! ```bash
//! DASHSCOPE_API_KEY=your_api_key cargo run --example qwen_streaming_with_mcp --features rmcp
//! ```

use anyhow::Result;
use rig::agent::stream_to_stdout;
use rig::prelude::*;
use rig::{completion::ToolDefinition, providers, streaming::StreamingPrompt, tool::Tool};

use rmcp::{
    model::{ClientCapabilities, ClientInfo, Implementation, Tool as McpTool},
    transport::{StreamableHttpClientTransport},
    transport::streamable_http_client::StreamableHttpClientTransportConfig,
    ServiceExt,
};
use futures::StreamExt;
#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    println!("=== 通义千问 + MCP 服务器流式输出示例 ===\n");

    // 1. 创建通义千问客户端
    // let client: qwen::Client = qwen::Client::from_env();

    // 2. 配置 MCP 服务器连接
    let mcp_server_url = "http://127.0.0.1:3001/mcp".to_string();
    let mcp_api_key = "tk_zaEVQtzrfFIXKh7EnBoja8KnGIfjV0T8".to_string();

    // 使用StreamableHttpClientTransportConfig添加Authorization头
    tracing::info!("Connecting to MCP server at: {}", mcp_server_url);
    tracing::info!("Using MCP Authorization Bearer token");

    let config = StreamableHttpClientTransportConfig::with_uri(mcp_server_url)
        .auth_header(mcp_api_key);

    let transport = StreamableHttpClientTransport::from_config(config);

    let client_info = ClientInfo {
        protocol_version: Default::default(),
        capabilities: ClientCapabilities::default(),
        client_info: Implementation {
            name: "Qwen-MCP client".to_string(),
            title: None,
            version: "0.0.1".to_string(),
            website_url: None,
            icons: None,
        },
    };

    // 3. 连接到 MCP 服务器
    let mcp_client = client_info.serve(transport).await.map_err(|e| {
        anyhow::anyhow!("Failed to connect to MCP server: {}", e)
    })?;

    // 列出 MCP 服务器提供的工具
    let tools_result = mcp_client
        .list_tools(Default::default())
        .await
        .map_err(|e| {
            anyhow::anyhow!("Failed to list MCP tools: {}", e)
        })?;
    let tools: Vec<McpTool> = tools_result.tools;

    // tracing::info!("Available MCP tools: {:?}", tools.iter().map(|t| &t.name).collect::<Vec<_>>());
    
    // 5. 创建 Qwen Agent 并添加 MCP 工具支持
    // let mut agent_builder: rig::agent::AgentBuilderSimple<providers::qwen::CompletionModel> = providers::qwen::Client::from_env()
    let api_key = "sk-348d7ca647714c52aca12ea106cfa895";
    let mut agent_builder = providers::qwen::Client::new_with_api_key(&api_key)
        .agent("qwen-plus")
        .preamble(
            "你是一个智能助手，可以连接到 MCP 服务器来使用各种工具。
            当用户请求需要调用外部服务或执行特定功能时，你会自动选择合适的 MCP 工具来完成任务。

            请根据用户的需求，判断是否需要调用 MCP 工具，如果需要则调用相应工具并提供准确的结果。"
        )
        .temperature(0.7)
        .tool(rig::tools::ThinkTool)
        .tool(rig::tools::ListTasks);
        // .rmcp_tools(tools, mcp_client.peer().to_owned());

    let agent = agent_builder.build();

    let mut stream = agent.stream_prompt("列出我的任务").await;
    let res = stream_to_stdout(&mut stream).await?;

    println!("Token usage response: {usage:?}", usage = res.usage());
    println!("Final text response: {message:?}", message = res.response());
    
    // // 直接处理流式输出
    // let mut response_text = String::new();
    // println!("AI 回答（流式）：");

    // while let Some(chunk_result) = stream.next().await {
    //     match chunk_result {
    //         Ok(chunk) => {
    //             match chunk {
    //                 rig::agent::MultiTurnStreamItem::StreamItem(rig::streaming::StreamedAssistantContent::Text(text)) => {
    //                     print!("{}", text.text);
    //                     response_text.push_str(&text.text);
    //                 }
    //                 rig::agent::MultiTurnStreamItem::StreamItem(rig::streaming::StreamedAssistantContent::ToolCall(tool_call)) => {
    //                     println!("\n[工具调用] {}: {}({})",
    //                         tool_call.id,
    //                         tool_call.function.name,
    //                         tool_call.function.arguments);
    //                 }
    //                 rig::agent::MultiTurnStreamItem::StreamItem(rig::streaming::StreamedAssistantContent::Reasoning(reasoning)) => {
    //                     println!("\n[推理] {}", reasoning.reasoning.join(" "));
    //                 }
    //                 rig::agent::MultiTurnStreamItem::FinalResponse(final_response) => {
    //                     println!("\n[最终响应]: {}", final_response.response());
    //                     response_text.push_str(&final_response.response());
    //                 }
    //                 _ => {
    //                     // 处理其他可能的变体
    //                     println!("\n[其他流式内容]");
    //                 }
    //             }
    //         }
    //         Err(e) => {
    //             eprintln!("流式输出错误: {}", e);
    //             break;
    //         }
    //     }
    // }
    Ok(())
}