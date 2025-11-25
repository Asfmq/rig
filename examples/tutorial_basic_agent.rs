//! 基础 Agent 示例
//! 
//! 演示如何创建和使用简单的 Agent

use rig::prelude::*;
use rig::completion::Prompt;
use rig::providers::openai;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .init();

    // 创建 OpenAI 客户端
    let client = openai::Client::from_env();

    println!("=== 示例 1: 最简单的 Agent ===");
    let simple_agent = client
        .agent("gpt-4o")
        .preamble("你是一个友好的助手，用简洁的语言回答问题。")
        .build();

    let response = simple_agent.prompt("什么是 Rust 编程语言？").await?;
    println!("回复: {}\n", response);

    println!("=== 示例 2: 带有上下文的 Agent ===");
    let context_agent = client
        .agent("gpt-4o")
        .preamble("你是公司的客服代表。")
        .context("公司名称：科技创新有限公司")
        .context("成立时间：2020年")
        .context("主营业务：人工智能解决方案")
        .context("客服时间：周一至周五 9:00-18:00")
        .build();

    let response = context_agent
        .prompt("请介绍一下你们公司")
        .await?;
    println!("回复: {}\n", response);

    println!("=== 示例 3: 设置温度参数 ===");
    
    // 低温度 - 更确定性的回答
    let deterministic_agent = client
        .agent("gpt-4o")
        .preamble("你是一个数学老师。")
        .temperature(0.2)
        .build();

    let response = deterministic_agent
        .prompt("2 + 2 等于多少？")
        .await?;
    println!("确定性回复: {}\n", response);

    // 高温度 - 更有创意的回答
    let creative_agent = client
        .agent("gpt-4o")
        .preamble("你是一个创意作家。")
        .temperature(0.9)
        .build();

    let response = creative_agent
        .prompt("用一句话描述月亮")
        .await?;
    println!("创意回复: {}\n", response);

    println!("=== 示例 4: 限制最大 Token 数 ===");
    let concise_agent = client
        .agent("gpt-4o")
        .preamble("简洁地回答问题。")
        .max_tokens(100)
        .build();

    let response = concise_agent
        .prompt("解释什么是机器学习")
        .await?;
    println!("简洁回复（最多 100 tokens）: {}\n", response);

    Ok(())
}

