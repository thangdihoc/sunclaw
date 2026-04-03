mod tui;

use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::*;
use dialoguer::{theme::ColorfulTheme, Input, Password, Select};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use sysinfo::System;
use serde::{Deserialize, Serialize};

use sunclaw_app::{build_runtime, RuntimeConfig};
use sunclaw_core::AgentContext;
use sunclaw_telegram::TelegramBridge;
use tui::run_tui;

#[derive(Parser, Debug)]
#[command(name = "sunclaw")]
#[command(author = "Sunclaw Team")]
#[command(version = "0.1.0")]
#[command(about = "Hệ thống AI Agent Hiệu năng cao (Rust)", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Kiểm tra tình trạng hệ thống và cấu hình
    #[arg(short, long)]
    doctor: bool,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Thiết lập cấu hình ban đầu
    Onboard,
    /// Chạy trực tiếp chế độ Telegram
    Telegram,
    /// Chat trực tiếp trên Terminal (Mặc định)
    Chat,
}

#[derive(Serialize, Deserialize, Default, Debug)]
struct Config {
    provider: String,
    api_key: String,
    tavily_key: Option<String>,
    tele_token: Option<String>,
}

fn get_config_path() -> PathBuf {
    let mut path = home::home_dir().unwrap_or_else(|| PathBuf::from("."));
    path.push(".sunclaw");
    if !path.exists() {
        fs::create_dir_all(&path).ok();
    }
    path.push("config.toml");
    path
}

fn load_config() -> Result<Config> {
    let path = get_config_path();
    if path.exists() {
        let content = fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    } else {
        // Fallback to .env for backward compatibility
        let _ = dotenvy::dotenv();
        Ok(Config {
            provider: std::env::var("SUNCLAW_PROVIDER").unwrap_or_else(|_| "openrouter".to_string()),
            api_key: std::env::var("SUNCLAW_API_KEY").unwrap_or_default(),
            tavily_key: std::env::var("TAVILY_API_KEY").ok(),
            tele_token: std::env::var("TELEGRAM_BOT_TOKEN").ok(),
        })
    }
}

fn save_config(config: &Config) -> Result<()> {
    let path = get_config_path();
    let content = toml::to_string_pretty(config)?;
    fs::write(path, content)?;
    Ok(())
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

async fn run_onboard(theme: &ColorfulTheme) -> Result<()> {
    println!("{}", ">>> PHẦN THIẾT LẬP CẤU HÌNH (ONBOARDING) <<<".bright_magenta().bold());

    // 1. Chọn Nhà cung cấp AI
    let providers = &["OpenRouter", "OpenAI", "Anthropic", "Google Gemini"];
    let provider_idx = Select::with_theme(theme)
        .with_prompt("Chọn nhà cung cấp AI mặc định")
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
        .with_prompt("Nhập Tavily API Key (Dùng cho Web Search, để trống nếu không dùng)")
        .allow_empty_password(true)
        .interact()?;

    // 4. Nhập Telegram Token
    let tele_token: String = Password::with_theme(theme)
        .with_prompt("Nhập Telegram Bot Token (Để trống nếu không dùng)")
        .allow_empty_password(true)
        .interact()?;

    let config = Config {
        provider: provider_name,
        api_key,
        tavily_key: if tavily_key.is_empty() { None } else { Some(tavily_key) },
        tele_token: if tele_token.is_empty() { None } else { Some(tele_token) },
    };

    save_config(&config)?;
    println!("\n{}", format!("✅ Đã lưu cấu hình vào: {:?}", get_config_path()).green().bold());
    
    Ok(())
}

fn run_doctor() -> Result<()> {
    println!("{}", "🔍 Đang kiểm tra hệ thống Sunclaw...".bright_cyan().bold());
    println!();

    // 1. Kiểm tra file config
    let config_path = get_config_path();
    if config_path.exists() {
        println!("✅ File cấu hình: {} ({:?})", "Tìm thấy".green(), config_path);
    } else if Path::new(".env").exists() {
        println!("⚠️  Cấu hình: {} (Đang dùng .env, khuyên dùng --onboard để đồng bộ)".yellow());
    } else {
        println!("❌ Cấu hình: {}", "Không tìm thấy (Cần chạy sunclaw onboard)".red());
    }

    // 2. Kiểm tra Database
    let db_path = "sunclaw.db";
    if Path::new(db_path).exists() {
        println!("✅ Database: {} ({})", "Kết nối tốt".green(), db_path);
    } else {
        println!("⚠️  Database: {} (Sẽ được tạo khi chạy Agent)", "Chưa khởi tạo".yellow());
    }

    println!();
    println!("{}", "--- Hoàn tất kiểm tra ---".bright_black());
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let theme = ColorfulTheme::default();

    show_header();
    show_system_info();

    if cli.doctor {
        run_doctor()?;
        return Ok(());
    }

    match cli.command {
        Some(Commands::Onboard) => {
            run_onboard(&theme).await?;
            return Ok(());
        }
        Some(Commands::Telegram) => {
             let config = load_config()?;
             if config.api_key.is_empty() {
                 println!("{}", "❌ Chưa cấu hình! Vui lòng chạy 'sunclaw onboard'.".red());
                 return Ok(());
             }
             start_telegram(config, &theme).await?;
        }
        Some(Commands::Chat) | None => {
            let config = load_config()?;
            if config.api_key.is_empty() {
                 println!("{}", "❌ Chưa cấu hình! Vui lòng chạy 'sunclaw onboard'.".red());
                 return Ok(());
            }
            
            // Nếu người dùng không chỉ định lệnh và file config chưa tồn tại, gợi ý onboard
            if !get_config_path().exists() && !Path::new(".env").exists() {
                 run_onboard(&theme).await?;
            }

            start_terminal_chat(config, &theme).await?;
        }
    }

    Ok(())
}

async fn start_terminal_chat(config: Config, _theme: &ColorfulTheme) -> Result<()> {
    // Mặc định chọn model
    let model_id = match config.provider.as_str() {
        "openrouter" => "deepseek/deepseek-chat",
        "openai" => "gpt-4o",
        "anthropic" => "claude-3-5-sonnet-20240620",
        "googlegemini" => "gemini-1.5-pro",
        _ => "default",
    }.to_string();

    let runtime_config = RuntimeConfig {
        provider: config.provider,
        model_id,
        api_key: config.api_key,
        tavily_key: config.tavily_key,
    };

    let runtime = Arc::new(build_runtime(Some(runtime_config)).await);

    println!("\n--- 🗨️ Đang khởi động Sunclaw TUI (Ratatui) ---");
    run_tui(runtime).await.map_err(|e| anyhow::anyhow!(e))?;
    
    Ok(())
}

async fn start_telegram(config: Config, theme: &ColorfulTheme) -> Result<()> {
    let token = config.tele_token.unwrap_or_else(|| {
        Password::with_theme(theme)
            .with_prompt("Nhập Telegram Bot Token")
            .interact().expect("Cần Token để chạy Telegram!")
    });

    if token.is_empty() {
         println!("{}", "❌ Thiếu Telegram Token! Vui lòng cấu hình qua 'sunclaw onboard'.".red());
         return Ok(());
    }

    // Tinh chỉnh runtime config cho Telegram
    let model_id = match config.provider.as_str() {
        "openrouter" => "deepseek/deepseek-chat",
        "openai" => "gpt-4o",
        "anthropic" => "claude-3-5-sonnet-20240620",
        "googlegemini" => "gemini-1.5-pro",
        _ => "default",
    }.to_string();

    let runtime_config = RuntimeConfig {
        provider: config.provider,
        model_id,
        api_key: config.api_key,
        tavily_key: config.tavily_key,
    };

    let runtime = Arc::new(build_runtime(Some(runtime_config)).await);

    println!("{}", "🚀 Đang khởi động Telegram Bot...".bright_green());
    use sunclaw_core::Bridge;
    let bridge = TelegramBridge::new(runtime, token, None);
    bridge.start().await.map_err(|e| anyhow::anyhow!(e))?;
    Ok(())
}
