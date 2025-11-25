//! 高级 Agent 编排示例
//! 
//! 演示高级编排技术：
//! 1. Pipeline 链式处理
//! 2. 并行执行
//! 3. 路由模式
//! 4. 复杂工作流

use rig::prelude::*;
use rig::pipeline::{self, Op, TryOp, passthrough};
use rig::pipeline::agent_ops::extract;
use rig::{parallel};
use rig::providers::openai::Client;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

// ============= 数据结构 =============

#[derive(Deserialize, JsonSchema, Serialize, Debug, Clone)]
struct ContentIdea {
    title: String,
    description: String,
    target_audience: String,
}

#[derive(Deserialize, JsonSchema, Serialize, Debug)]
struct ContentIdeas {
    ideas: Vec<ContentIdea>,
}

#[derive(Deserialize, JsonSchema, Serialize, Debug)]
struct ContentPiece {
    title: String,
    content: String,
}

#[derive(Deserialize, JsonSchema, Serialize, Debug)]
struct QualityScore {
    score: f32,
    feedback: String,
}

#[derive(Deserialize, JsonSchema, Serialize, Debug)]
struct SentimentScore {
    score: f32,
    sentiment: String,
}

#[derive(Deserialize, JsonSchema, Serialize, Debug)]
struct SEOScore {
    score: f32,
    recommendations: Vec<String>,
}

// ============= Pipeline 示例 =============

async fn example_pipeline_chain(client: &Client) -> Result<(), anyhow::Error> {
    println!("=== 示例 1: Pipeline 链式处理 ===\n");
    println!("创建一个内容创作 Pipeline：创意生成 -> 内容撰写 -> 质量评估\n");

    // Agent 1: 创意生成器
    let idea_generator = client
        .agent("gpt-4o")
        .preamble(
            "你是创意生成器。根据主题生成 3 个有创意的内容想法。\
            每个想法要包含标题、描述和目标受众。保持简洁。"
        )
        .temperature(0.8)
        .build();

    // Agent 2: 内容撰写者
    let content_writer = client
        .agent("gpt-4o")
        .preamble(
            "你是内容撰写者。根据创意生成高质量的内容。\
            内容应该引人入胜、信息丰富且结构清晰。"
        )
        .temperature(0.7)
        .build();

    // Agent 3: 质量评估者
    let quality_evaluator = client
        .agent("gpt-4o")
        .preamble(
            "你是内容质量评估者。评估内容的质量，提供建设性的反馈。\
            考虑：清晰度、准确性、可读性、吸引力。"
        )
        .temperature(0.3)
        .build();

    // 创建 Pipeline
    let content_pipeline = pipeline::new()
        .prompt(idea_generator)
        .map(|ideas| {
            println!("生成的创意:\n{}\n", ideas);
            format!("请根据这些创意撰写第一个创意的内容: {}", ideas)
        })
        .prompt(content_writer)
        .map(|content| {
            println!("撰写的内容:\n{}\n", content);
            format!("请评估这篇内容的质量: {}", content)
        })
        .prompt(quality_evaluator);

    let final_result = content_pipeline
        .try_call("人工智能在教育领域的应用")
        .await?;

    println!("最终评估:\n{}\n", final_result);

    Ok(())
}

async fn example_parallel_execution(client: &Client) -> Result<(), anyhow::Error> {
    println!("=== 示例 2: 并行执行多个评估 ===\n");

    // 创建多个评估 Agent
    let quality_agent = client
        .extractor::<QualityScore>("gpt-4o")
        .preamble(
            "评估文本质量（0-1）。考虑清晰度、语法、结构。\
            返回分数和简短反馈。"
        )
        .build();

    let sentiment_agent = client
        .extractor::<SentimentScore>("gpt-4o")
        .preamble(
            "分析文本情感（0-1，0=负面，1=正面）。\
            返回分数和情感类型（正面/中性/负面）。"
        )
        .build();

    let seo_agent = client
        .extractor::<SEOScore>("gpt-4o")
        .preamble(
            "评估文本的 SEO 友好度（0-1）。\
            返回分数和最多 3 个改进建议。"
        )
        .build();

    // 创建并行 Pipeline
    let parallel_pipeline = pipeline::new()
        .chain(parallel!(
            passthrough(),
            extract(quality_agent),
            extract(sentiment_agent),
            extract(seo_agent)
        ))
        .map(|(text, quality, sentiment, seo)| {
            let quality = quality.unwrap();
            let sentiment = sentiment.unwrap();
            let seo = seo.unwrap();

            format!(
                "原文: {}\n\n\
                === 评估结果 ===\n\
                质量分数: {:.2}/1.0\n\
                反馈: {}\n\n\
                情感分数: {:.2}/1.0\n\
                情感类型: {}\n\n\
                SEO 分数: {:.2}/1.0\n\
                建议: {:?}",
                text,
                quality.score,
                quality.feedback,
                sentiment.score,
                sentiment.sentiment,
                seo.score,
                seo.recommendations
            )
        });

    let test_text = "Rust 是一种系统编程语言，它专注于安全性、速度和并发性。\
                     Rust 的所有权系统使其能够保证内存安全，而无需垃圾回收。\
                     这使得 Rust 成为构建高性能应用程序的理想选择。";

    let result = parallel_pipeline.call(test_text).await;

    println!("{}\n", result);

    Ok(())
}

async fn example_routing_pattern(client: &Client) -> Result<(), anyhow::Error> {
    println!("=== 示例 3: 路由模式 ===\n");

    // 创建分类器 Agent
    let classifier = client
        .agent("gpt-4o")
        .preamble(
            "分类用户请求到以下类别之一：\n\
            - technical: 技术问题\n\
            - creative: 创意任务\n\
            - analytical: 数据分析\n\
            - general: 一般问题\n\n\
            只返回类别名称，不要其他内容。"
        )
        .temperature(0.1)
        .build();

    // 创建专门的处理 Agent
    let technical_agent = client
        .agent("gpt-4o")
        .preamble("你是技术专家，提供详细的技术解答。")
        .temperature(0.3)
        .build();

    let creative_agent = client
        .agent("gpt-4o")
        .preamble("你是创意专家，提供创新的想法和方案。")
        .temperature(0.9)
        .build();

    let analytical_agent = client
        .agent("gpt-4o")
        .preamble("你是数据分析专家，提供深入的分析和洞察。")
        .temperature(0.3)
        .build();

    let general_agent = client
        .agent("gpt-4o")
        .preamble("你是通用助手，回答各类问题。")
        .temperature(0.5)
        .build();

    // 创建路由 Pipeline
    let router = pipeline::new()
        .chain(parallel!(
            passthrough(),
            pipeline::new().prompt(classifier)
        ))
        .map(|(original_query, category): (String, String)| {
            println!("查询: {}", original_query);
            println!("分类: {}\n", category.trim());
            (original_query, category)
        });

    // 测试不同类型的查询
    let test_queries = vec![
        "解释 Rust 的生命周期是如何工作的",
        "给我的咖啡店想 5 个创意营销活动",
        "分析电商网站的用户留存率数据",
        "今天天气怎么样？",
    ];

    for query in test_queries {
        let (original, category) = router.try_call(query).await?;
        let category = category.trim();

        // 根据分类选择合适的 Agent
        let response = match category {
            "technical" => technical_agent.prompt(&original).await?,
            "creative" => creative_agent.prompt(&original).await?,
            "analytical" => analytical_agent.prompt(&original).await?,
            _ => general_agent.prompt(&original).await?,
        };

        println!("回复: {}\n", response);
        println!("{}", "=".repeat(60));
        println!();
    }

    Ok(())
}

async fn example_complex_workflow(client: &Client) -> Result<(), anyhow::Error> {
    println!("=== 示例 4: 复杂工作流 - 文章发布系统 ===\n");

    // 步骤 1: 生成内容创意
    let idea_agent = client
        .extractor::<ContentIdeas>("gpt-4o")
        .preamble("根据主题生成 3 个内容创意。")
        .temperature(0.8)
        .build();

    println!("步骤 1: 生成创意...");
    let ideas = idea_agent
        .extract("区块链技术在供应链管理中的应用")
        .await?;
    
    println!("生成了 {} 个创意:\n", ideas.ideas.len());
    for (i, idea) in ideas.ideas.iter().enumerate() {
        println!("  {}. {} - {}", i + 1, idea.title, idea.description);
    }
    println!();

    // 步骤 2: 为第一个创意撰写内容
    let writer_agent = client
        .extractor::<ContentPiece>("gpt-4o")
        .preamble("根据创意撰写详细的文章内容（200-300字）。")
        .temperature(0.7)
        .build();

    let first_idea = &ideas.ideas[0];
    println!("步骤 2: 撰写文章 '{}'...", first_idea.title);
    
    let content = writer_agent
        .extract(&format!(
            "标题: {}\n描述: {}\n目标受众: {}",
            first_idea.title,
            first_idea.description,
            first_idea.target_audience
        ))
        .await?;
    
    println!("文章已撰写（{} 字符）\n", content.content.len());

    // 步骤 3: 并行评估
    let quality_agent = client
        .extractor::<QualityScore>("gpt-4o")
        .preamble("评估内容质量（0-1）。")
        .build();

    let seo_agent = client
        .extractor::<SEOScore>("gpt-4o")
        .preamble("评估 SEO 友好度（0-1）。")
        .build();

    println!("步骤 3: 并行评估质量和 SEO...");

    let (quality_result, seo_result) = tokio::try_join!(
        quality_agent.extract(&content.content),
        seo_agent.extract(&content.content)
    )?;

    println!("  质量分数: {:.2}", quality_result.score);
    println!("  SEO 分数: {:.2}\n", seo_result.score);

    // 步骤 4: 决策 - 是否需要改进
    let min_quality = 0.7;
    let min_seo = 0.6;

    if quality_result.score >= min_quality && seo_result.score >= min_seo {
        println!("✓ 内容通过质量检查，可以发布！");
    } else {
        println!("✗ 内容需要改进：");
        
        if quality_result.score < min_quality {
            println!("  - 质量: {}", quality_result.feedback);
        }
        
        if seo_result.score < min_seo {
            println!("  - SEO: {:?}", seo_result.recommendations);
        }

        // 步骤 5: 改进内容
        println!("\n步骤 4: 改进内容...");
        let editor_agent = client
            .agent("gpt-4o")
            .preamble("根据反馈改进文章内容。")
            .build();

        let improvement_prompt = format!(
            "请改进以下文章：\n\n{}\n\n质量反馈: {}\n\nSEO 建议: {:?}",
            content.content,
            quality_result.feedback,
            seo_result.recommendations
        );

        let improved = editor_agent.prompt(&improvement_prompt).await?;
        println!("内容已改进！\n");
        println!("改进后的内容:\n{}", improved);
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
    example_pipeline_chain(&client).await?;
    println!("\n{}\n", "=".repeat(80));

    example_parallel_execution(&client).await?;
    println!("\n{}\n", "=".repeat(80));

    example_routing_pattern(&client).await?;
    println!("\n{}\n", "=".repeat(80));

    example_complex_workflow(&client).await?;

    println!("\n所有高级示例完成！");

    Ok(())
}

