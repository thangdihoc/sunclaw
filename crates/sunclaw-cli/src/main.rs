use anyhow::Result;
use dialoguer::{Input, Password, Select, theme::ColorfulTheme};
use std::sync::Arc;
use sunclaw_app::{build_runtime, RuntimeConfig};
use sunclaw_core::AgentContext;
use sunclaw_telegram::TelegramBridge;

#[tokio::main]
async fn main() -> Result<()> {
    println!("=== 🦀 Sunclaw v0.1: AI Agent Setup ===");

    let theme = ColorfulTheme::default();

    // 1. Chọn Nhà cung cấp AI
    let providers = &["OpenRouter", "OpenAI", "Anthropic", "Google Gemini"];
    let provider_idx = Select::with_theme(&theme)
        .with_prompt("Chọn nhà cung cấp AI")
        .items(providers)
        .default(0)
        .interact()?;
    
    let provider_name = providers[provider_idx].to_lowercase().replace(" ", "");

    // 2. Nhập API Key cho Provider đã chọn
    let api_key: String = Password::with_theme(&theme)
        .with_prompt(format!("Nhập API Key cho {}", providers[provider_idx]))
        .interact()?;

    // 3. Chọn Model dựa trên Provider
    let models = match provider_name.as_str() {
        "openrouter" => vec!["deepseek/deepseek-chat", "anthropic/claude-3.5-sonnet", "google/gemini-pro-1.5", "meta-llama/llama-3.1-405b"],
        "openai" => vec!["gpt-4o", "gpt-4-turbo", "gpt-3.5-turbo"],
        "anthropic" => vec!["claude-3-5-sonnet-20240620", "claude-3-opus-20240229"],
        "googlegemini" => vec!["gemini-1.5-pro", "gemini-1.5-flash"],
        _ => vec!["default"],
    };

    let model_idx = Select::with_theme(&theme)
        .with_prompt("Chọn Model AI")
        .items(&models)
        .default(0)
        .interact()?;
    
    let model_id = models[model_idx].to_string();

    // 4. Nhập Tavily Key
    let tavily_key: String = Password::with_theme(&theme)
        .with_prompt("Nhập Tavily API Key (Tìm kiếm Web - để trống nếu không dùng)")
        .allow_empty_password(true)
        .interact()?;

    let config = RuntimeConfig {
        provider: provider_name,
        model_id,
        api_key,
        tavily_key: if tavily_key.is_empty() { None } else { Some(tavily_key) },
    };

    let runtime = Arc::new(build_runtime(Some(config)).await);

    // 2. Select Run Mode
    let modes = &["Chat trên Terminal", "Chạy Telegram Bot"];
    let selection = Select::with_theme(&theme)
        .with_prompt("Chọn chế độ hoạt động")
        .items(modes)
        .default(0)
        .interact()?;

    match selection {
        0 => {
            // Terminal Chat Mode
            println!("\n--- Đang bắt đầu Terminal Chat (Gõ 'quit' để thoát) ---");
            loop {
                let input: String = Input::with_theme(&theme)
                    .with_prompt("Bạn")
                    .interact_text()?;

                if input == "quit" || input == "exit" {
                    break;
                }

                let ctx = AgentContext {
                    trace_id: "cli-chat".to_string(),
                    skill: None,
                    model_profile: Some("default".to_string()),
                    role: None,
                    max_tokens: None,
                };

                match runtime.run_once(&ctx, &input).await {
                    Ok(outcome) => {
                        println!("\n[AI]: {}\n(turns={}, tools={})\n", outcome.output, outcome.turns, outcome.tool_calls);
                    }
                    Err(e) => {
                        println!("\n[Error]: {:?}\n", e);
                    }
                }
            }
        }
        1 => {
            // Telegram Bot Mode
            let token: String = Password::with_theme(&theme)
                .with_prompt("Nhập Telegram Bot Token")
                .interact()?;

            let chat_id_str: String = Input::with_theme(&theme)
                .with_prompt("Nhập Authorized Chat ID (ID của bạn - để trống nếu cho phép tất cả)")
                .allow_empty(true)
                .interact_text()?;

            let chat_id: Option<i64> = if chat_id_str.is_empty() {
                None
            } else {
                Some(chat_id_str.parse().expect("Chat ID phải là một con số!"))
            };

            use sunclaw_core::Bridge;
            let bridge = TelegramBridge::new(runtime, token, chat_id);
            bridge.start().await.map_err(|e| anyhow::anyhow!(e))?;
        }
        _ => unreachable!(),
    }

    Ok(())
}
