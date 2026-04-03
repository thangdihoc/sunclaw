use anyhow::Result;
use clap::Parser;
use colored::*;
use dialoguer::{theme::ColorfulTheme, Input, Password, Select};
use std::fs;
use std::path::Path;
use std::sync::Arc;
use sysinfo::System;

use sunclaw_app::{build_runtime, RuntimeConfig};
use sunclaw_core::AgentContext;
use sunclaw_telegram::TelegramBridge;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Kích hoạt chế độ thiết lập (Cấu hình lại API Keys)
    #[arg(short, long)]
    setup: bool,

    /// Chạy trực tiếp chế độ Telegram (nếu đã cấu hình)
    #[arg(short, long)]
    telegram: bool,
}

fn show_header() {
    let art = r#"
 ██████  ██    ██ ███    ██  ██████ ██       █████  ██     ██ 
██       ██    ██ ████   ██ ██      ██      ██   ██ ██     ██ 
 ██████  ██    ██ ██ ██  ██ ██      ██      ███████ ██  █  ██ 
      ██ ██    ██ ██  ██ ██ ██      ██      ██   ██ ██ ███ ██ 
 ██████   ██████  ██   ████  ██████ ███████ ██   ██  ███ ███  
    "#;
    println!("{}", art.cyan().bold());
    println!("{}", "--- Hệ thống AI Agent Hiệu năng cao (Rust) ---".yellow());
    println!();
}

fn show_system_info() {
    let mut sys = System::new_all();
    sys.refresh_all();

    println!("{}", "╔════════════════════════════════════════════════════════════╗".bright_black());
    println!("║ {:^58} ║", "THÔNG TIN HỆ THỐNG".bold().bright_white());
    println!("{}", "╠════════════════════════════════════════════════════════════╣".bright_black());
    
    let cpu_brand = sys.cpus().get(0).map(|c| c.brand()).unwrap_or("Unknown");
    println!("║ {}: {:<48} ║", "CPU".green(), cpu_brand.trim());
    
    let total_mem = sys.total_memory() / 1024 / 1024 / 1024;
    println!("║ {}: {:<48} ║", "RAM".green(), format!("{} GB", total_mem));
    
    let os = System::long_os_version().unwrap_or_else(|| "Unknown".to_string());
    println!("║ {}: {:<49} ║", "OS".green(), os);
    
    println!("{}", "╚════════════════════════════════════════════════════════════╝".bright_black());
    println!();
}

async fn run_setup(theme: &ColorfulTheme) -> Result<()> {
    println!("{}", ">>> PHẦN THIẾT LẬP CẤU HÌNH <<<".bright_magenta().bold());

    // 1. Chọn Nhà cung cấp AI
    let providers = &["OpenRouter", "OpenAI", "Anthropic", "Google Gemini"];
    let provider_idx = Select::with_theme(theme)
        .with_prompt("Chọn nhà cung cấp AI")
        .items(providers)
        .default(0)
        .interact()?;
    
    let provider_name = providers[provider_idx].to_lowercase().replace(" ", "");

    // 2. Nhập API Key cho Provider
    let api_key: String = Password::with_theme(theme)
        .with_prompt(format!("Nhập API Key cho {}", providers[provider_idx]))
        .interact()?;

    // 3. Nhập Tavily Key
    let tavily_key: String = Password::with_theme(theme)
        .with_prompt("Nhập Tavily API Key (Để trống nếu không dùng)")
        .allow_empty_password(true)
        .interact()?;

    // 4. Nhập Telegram Token
    let tele_token: String = Password::with_theme(theme)
        .with_prompt("Nhập Telegram Bot Token (Để trống nếu không dùng)")
        .allow_empty_password(true)
        .interact()?;

    // Lưu vào file .env
    let mut env_content = format!(
        "SUNCLAW_PROVIDER={}\nSUNCLAW_API_KEY={}\n",
        provider_name, api_key
    );
    if !tavily_key.is_empty() {
        env_content.push_str(&format!("TAVILY_API_KEY={}\n", tavily_key));
    }
    if !tele_token.is_empty() {
        env_content.push_str(&format!("TELEGRAM_BOT_TOKEN={}\n", tele_token));
    }

    fs::write(".env", env_content)?;
    println!("\n{}", "✅ Đã lưu cấu hình vào file .env!".green().bold());
    
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let theme = ColorfulTheme::default();

    show_header();
    show_system_info();

    // Load .env
    let _ = dotenvy::dotenv();

    // Kiểm tra nếu cần setup
    if args.setup || !Path::new(".env").exists() {
        run_setup(&theme).await?;
        // Reload .env sau khi setup
        let _ = dotenvy::dotenv();
    }

    // Lấy cấu hình từ môi trường
    let provider = std::env::var("SUNCLAW_PROVIDER").unwrap_or_else(|_| "openrouter".to_string());
    let api_key = std::env::var("SUNCLAW_API_KEY").unwrap_or_default();
    let tavily_key = std::env::var("TAVILY_API_KEY").ok();
    let tele_token = std::env::var("TELEGRAM_BOT_TOKEN").ok();

    if api_key.is_empty() {
        println!("{}", "❌ Thiếu API Key! Vui lòng chạy với flag --setup để cấu hình.".red());
        return Ok(());
    }

    // Mặc định chọn model cho nhanh nếu đã có config
    let model_id = match provider.as_str() {
        "openrouter" => "deepseek/deepseek-chat",
        "openai" => "gpt-4o",
        "anthropic" => "claude-3-5-sonnet-20240620",
        "googlegemini" => "gemini-1.5-pro",
        _ => "default",
    }.to_string();

    let config = RuntimeConfig {
        provider,
        model_id,
        api_key,
        tavily_key,
    };

    let runtime = Arc::new(build_runtime(Some(config)).await);

    // Chọn chế độ
    let selection = if args.telegram { 1 } else {
        let modes = &["Chat trên Terminal", "Chạy Telegram Bot"];
        Select::with_theme(&theme)
            .with_prompt("Chọn chế độ hoạt động")
            .items(modes)
            .default(0)
            .interact()?
    };

    match selection {
        0 => {
            println!("\n--- 🗨️ Terminal Chat (Gõ 'quit' để thoát) ---");
            loop {
                let input: String = Input::with_theme(&theme)
                    .with_prompt("Bạn".cyan().to_string())
                    .interact_text()?;

                if input == "quit" || input == "exit" { break; }

                let ctx = AgentContext {
                    trace_id: "cli-chat".to_string(),
                    skill: None,
                    model_profile: Some("default".to_string()),
                    role: None,
                    max_tokens: None,
                };

                match runtime.run_once(&ctx, &input).await {
                    Ok(outcome) => {
                        println!("\n{} {}\n", "🤖 [AI]:".yellow().bold(), outcome.output);
                        println!("{}", format!("(Số lượt: {}, Công cụ đã dùng: {})", outcome.turns, outcome.tool_calls).bright_black());
                    }
                    Err(e) => println!("\n{} {:?}\n", "❌ [Lỗi]:".red(), e),
                }
            }
        }
        1 => {
            let token = tele_token.unwrap_or_else(|| {
                Password::with_theme(&theme)
                    .with_prompt("Nhập Telegram Bot Token")
                    .interact().expect("Cần Token để chạy Telegram!")
            });

            if token.is_empty() {
                 println!("{}", "❌ Thiếu Telegram Token! Vui lòng cấu hình qua --setup.".red());
                 return Ok(());
            }

            println!("{}", "🚀 Đang khởi động Telegram Bot...".bright_green());
            use sunclaw_core::Bridge;
            let bridge = TelegramBridge::new(runtime, token, None);
            bridge.start().await.map_err(|e| anyhow::anyhow!(e))?;
        }
        _ => unreachable!(),
    }

    Ok(())
}
