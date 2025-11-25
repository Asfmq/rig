use anyhow::Result;
use rig::prelude::*;
use rig::{providers, streaming::StreamingPrompt};
use futures::StreamExt;
use rig::agent::stream_to_stdout;
use rmcp::{
    model::{ClientCapabilities, ClientInfo, Implementation, Tool as McpTool},
    transport::{StreamableHttpClientTransport},
    transport::streamable_http_client::StreamableHttpClientTransportConfig,
    ServiceExt,
};



#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    tracing_subscriber::fmt().init();

    // 1. 连接到 MCP 服务器
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
            name: "Ollama-MCP client".to_string(),
            title: None,
            version: "0.0.1".to_string(),
            website_url: None,
            icons: None,
        },
    };

    // 连接到 MCP 服务器
    let mcp_client = client_info.serve(transport).await.map_err(|e| {
        anyhow::anyhow!("Failed to connect to MCP server: {}", e)
    })?;

    // 2. 获取服务器信息
    let server_info = mcp_client.peer_info();
    tracing::info!("Connected to MCP server: {server_info:#?}");

    // 列出 MCP 服务器提供的工具
    let tools_result = mcp_client
        .list_tools(Default::default())
        .await
        .map_err(|e| {
            anyhow::anyhow!("Failed to list MCP tools: {}", e)
        })?;
    let tools: Vec<McpTool> = tools_result.tools;

    // tracing::info!("Available MCP tools: {:?}", tools.iter().map(|t| &t.name).collect::<Vec<_>>());

    // 3. 创建 Ollama 客户端和 Agent
    let mut agent_builder = providers::ollama::Client::new()
        .agent("qwen3:4b")
        .preamble(
            "你是一个材料方向的助理，擅长数学计算和使用工具进行计算。
            ",
        )
        .max_tokens(1024);
        .tool(rig::tools::ThinkTool)
        .rmcp_tools(tools, mcp_client.peer().to_owned());

    let agent = agent_builder.build();


    let mut stream = agent.stream_prompt("列出我的任务").await;

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
    //                     println!("{}", reasoning.reasoning.join(" "));
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
    let res = stream_to_stdout(&mut stream).await?;
    println!("Token usage response: {usage:?}", usage = res.usage());
    println!("Final text response: {message:?}", message = res.response());
    Ok(())
}