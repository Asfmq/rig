//! Agent 工具使用示例
//! 
//! 演示如何创建自定义工具并在 Agent 中使用

use rig::prelude::*;
use rig::completion::{Prompt, ToolDefinition};
use rig::tool::Tool;
use serde::{Deserialize, Serialize};
use serde_json::json;

// ============= 工具定义 =============

// 计算器工具
#[derive(Deserialize, Serialize)]
struct Calculator;

#[derive(Deserialize)]
struct CalculatorArgs {
    x: f64,
    y: f64,
    operation: String,
}

#[derive(Debug, thiserror::Error)]
#[error("计算错误")]
struct MathError;

impl Tool for Calculator {
    const NAME: &'static str = "calculator";
    type Error = MathError;
    type Args = CalculatorArgs;
    type Output = f64;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        serde_json::from_value(json!({
            "name": "calculator",
            "description": "执行基本的数学运算：加法(add)、减法(subtract)、乘法(multiply)、除法(divide)",
            "parameters": {
                "type": "object",
                "properties": {
                    "x": {
                        "type": "number",
                        "description": "第一个数字"
                    },
                    "y": {
                        "type": "number",
                        "description": "第二个数字"
                    },
                    "operation": {
                        "type": "string",
                        "enum": ["add", "subtract", "multiply", "divide"],
                        "description": "要执行的操作"
                    }
                },
                "required": ["x", "y", "operation"]
            }
        }))
        .expect("Tool Definition")
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let result = match args.operation.as_str() {
            "add" => args.x + args.y,
            "subtract" => args.x - args.y,
            "multiply" => args.x * args.y,
            "divide" => {
                if args.y == 0.0 {
                    println!("[计算器] 错误: 除数不能为零");
                    return Err(MathError);
                }
                args.x / args.y
            }
            _ => {
                println!("[计算器] 错误: 未知操作 '{}'", args.operation);
                return Err(MathError);
            }
        };

        println!(
            "[计算器] 执行 {} {} {} = {}",
            args.x, args.operation, args.y, result
        );
        Ok(result)
    }
}

// 天气查询工具（模拟）
#[derive(Deserialize, Serialize)]
struct WeatherTool;

#[derive(Deserialize)]
struct WeatherArgs {
    city: String,
}

impl Tool for WeatherTool {
    const NAME: &'static str = "get_weather";
    type Error = anyhow::Error;
    type Args = WeatherArgs;
    type Output = String;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        serde_json::from_value(json!({
            "name": "get_weather",
            "description": "获取指定城市的当前天气信息",
            "parameters": {
                "type": "object",
                "properties": {
                    "city": {
                        "type": "string",
                        "description": "城市名称，例如：北京、上海"
                    }
                },
                "required": ["city"]
            }
        }))
        .expect("Tool Definition")
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        println!("[天气工具] 查询 {} 的天气", args.city);

        // 模拟天气数据
        let weather = match args.city.as_str() {
            "北京" => "晴天，气温 25°C，湿度 40%",
            "上海" => "多云，气温 28°C，湿度 65%",
            "深圳" => "小雨，气温 26°C，湿度 80%",
            _ => "多云，气温 22°C，湿度 50%",
        };

        Ok(format!("{} 的天气：{}", args.city, weather))
    }
}

// 单位转换工具
#[derive(Deserialize, Serialize)]
struct UnitConverter;

#[derive(Deserialize)]
struct ConvertArgs {
    value: f64,
    from_unit: String,
    to_unit: String,
}

impl Tool for UnitConverter {
    const NAME: &'static str = "convert_units";
    type Error = anyhow::Error;
    type Args = ConvertArgs;
    type Output = f64;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        serde_json::from_value(json!({
            "name": "convert_units",
            "description": "转换不同单位。支持：km_to_miles, miles_to_km, celsius_to_fahrenheit, fahrenheit_to_celsius",
            "parameters": {
                "type": "object",
                "properties": {
                    "value": {
                        "type": "number",
                        "description": "要转换的数值"
                    },
                    "from_unit": {
                        "type": "string",
                        "description": "原始单位"
                    },
                    "to_unit": {
                        "type": "string",
                        "description": "目标单位"
                    }
                },
                "required": ["value", "from_unit", "to_unit"]
            }
        }))
        .expect("Tool Definition")
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let conversion = format!("{}_{}", args.from_unit, args.to_unit);
        
        let result = match conversion.as_str() {
            "km_miles" => args.value * 0.621371,
            "miles_km" => args.value * 1.60934,
            "celsius_fahrenheit" => args.value * 9.0 / 5.0 + 32.0,
            "fahrenheit_celsius" => (args.value - 32.0) * 5.0 / 9.0,
            _ => {
                return Err(anyhow::anyhow!("不支持的单位转换: {}", conversion));
            }
        };

        println!(
            "[单位转换] {} {} 转换为 {} {}",
            args.value, args.from_unit, result, args.to_unit
        );

        Ok(result)
    }
}

// ============= 主程序 =============

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .with_target(false)
        .init();

    let client = rig::providers::openai::Client::from_env();

    println!("=== 示例 1: 单个工具 - 计算器 ===\n");
    let calculator_agent = client
        .agent("gpt-4o")
        .preamble("你是一个数学助手。使用计算器工具执行数学运算。")
        .tool(Calculator)
        .build();

    let response = calculator_agent
        .prompt("计算 123 加 456")
        .multi_turn(5)
        .await?;
    println!("Agent 回复: {}\n", response);

    println!("=== 示例 2: 复杂计算 ===\n");
    let response = calculator_agent
        .prompt("计算 (50 + 30) * 2 然后除以 4")
        .multi_turn(10)
        .await?;
    println!("Agent 回复: {}\n", response);

    println!("=== 示例 3: 多个工具 ===\n");
    let multi_tool_agent = client
        .agent("gpt-4o")
        .preamble(
            "你是一个多功能助手，可以执行计算、查询天气和转换单位。\
            根据用户的请求使用合适的工具。"
        )
        .tool(Calculator)
        .tool(WeatherTool)
        .tool(UnitConverter)
        .build();

    let response = multi_tool_agent
        .prompt("北京的天气怎么样？如果温度是 25 摄氏度，华氏度是多少？")
        .multi_turn(10)
        .await?;
    println!("Agent 回复: {}\n", response);

    println!("=== 示例 4: 组合使用多个工具 ===\n");
    let response = multi_tool_agent
        .prompt(
            "我要跑步 5 公里，这是多少英里？然后查一下上海的天气，看看适不适合跑步。"
        )
        .multi_turn(10)
        .await?;
    println!("Agent 回复: {}\n", response);

    Ok(())
}

