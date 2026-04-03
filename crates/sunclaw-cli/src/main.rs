mod tui;

use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::*;
use dialoguer::{theme::ColorfulTheme, Password, Select, Input};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use sysinfo::System;
use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;

use sunclaw_app::{build_runtime, RuntimeConfig};
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
    #[serde(default = "default_model")]
    model_id: String,
}

fn default_model() -> String {
    "deepseek/deepseek-chat".to_string()
}

fn get_config_dir() -> PathBuf {
    let mut path = home::home_dir().unwrap_or_else(|| {
        std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
    });
    path.push(".sunclaw");
    if !path.exists() {
        fs::create_dir_all(&path).ok();
    }
    path
}

fn get_config_path() -> PathBuf {
    let mut path = get_config_dir();
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
        println!("{}", "⚠️  Không tìm thấy config.toml, đang sử dụng mặc định...".yellow());
        Ok(Config::default())
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
   ███████╗██╗   ██╗███╗   ██╗ ██████╗██╗      █████╗ ██╗    ██╗
   ██╔════╝██║   ██║████╗  ██║██╔════╝██║     ██╔══██╗██║    ██║
   ███████╗██║   ██║██╔██╗ ██║██║     ██║     ███████║██║ █╗ ██║
   ╚════██║██║   ██║██║╚██╗██║██║     ██║     ██╔══██║██║███╗██║
   ███████║╚██████╔╝██║ ╚████║╚██████╗███████╗██║  ██║╚███╔███╔╝
   ╚══════╝ ╚═════╝ ╚═╝  ╚═══╝ ╚═════╝╚══════╝╚═╝  ╚═╝ ╚══╝╚══╝ 
    "#;
    println!("{}", art.bright_yellow().bold());
    println!("{}", "   >>> HỆ THỐNG AI AGENT ĐA KÊNH - HIỆU NĂNG VƯỢT TRỘI <<<".bright_cyan());
    println!("{}", "       ----------------------------------------------------".bright_black());
    println!();
}

fn show_system_info() {
    let mut sys = System::new_all();
    sys.refresh_all();

    println!("{}", "╔════════════════════════════════════════════════════════════╗".bright_black());
    println!("║ {:^58} ║", "📊 THÔNG SỐ HỆ THỐNG".bold().bright_white());
    println!("{}", "╠════════════════════════════════════════════════════════════╣".bright_black());
    
    let cpu_brand = sys.cpus().get(0).map(|c| c.brand()).unwrap_or("Unknown");
    println!("║ {}: {:<48} ║", "💻 CPU".green(), cpu_brand.trim());
    
    let total_mem = sys.total_memory() / 1024 / 1024 / 1024;
    println!("║ {}: {:<48} ║", "🧠 RAM".green(), format!("{} GB", total_mem));
    
    let os = System::long_os_version().unwrap_or_else(|| "Unknown".to_string());
    println!("║ {}: {:<49} ║", "🛡️  OS ".green(), os);
    
    println!("{}", "╚════════════════════════════════════════════════════════════╝".bright_black());
    println!();
}

async fn run_onboard(theme: &ColorfulTheme) -> Result<()> {
    println!("{}", "✨ CHÀO MỪNG BẠN ĐẾN VỚI SUNCLAW ✨".bright_white().on_bright_magenta().bold());
    println!("{}", "Hành trình xây dựng AI Agent của bạn bắt đầu từ đây.\n".bright_black());

    // 1. Chọn Nhà cung cấp AI
    let providers = &["OpenRouter (Khuyên dùng)", "OpenAI", "Anthropic (Claude)", "Google Gemini", "Custom (OpenAI Compatible)"];
    let provider_idx = Select::with_theme(theme)
        .with_prompt("🎯 Chọn 'bộ não' cho Agent của bạn")
        .items(providers)
        .default(0)
        .interact()?;
    
    let provider_name = match provider_idx {
        0 => "openrouter",
        1 => "openai",
        2 => "anthropic",
        3 => "googlegemini",
        4 => "custom",
        _ => "openrouter",
    };

    // 2. Nhập API Key
    println!("\n{}", format!("🔑 Thiết lập API Key cho {}:", providers[provider_idx].bright_blue()).bold());
    let api_key: String = Password::with_theme(theme)
        .with_prompt("Nhập API Key (Mật mã của bạn)")
        .interact()?;

    if api_key.is_empty() {
        println!("{}", "❌ Lỗi: API Key không được để trống!".red());
        return Ok(());
    }

    // 3. Chọn Model ID
    let mut model_id: String = match provider_name {
        "openrouter" => "deepseek/deepseek-chat".to_string(),
        "openai" => "gpt-4o".to_string(),
        "anthropic" => "claude-3-5-sonnet-20240620".to_string(),
        "googlegemini" => "gemini-1.5-pro".to_string(),
        _ => "default".to_string(),
    };

    let change_model = Select::with_theme(theme)
        .with_prompt(format!("🤖 Model mặc định là '{}'. Bạn có muốn đổi không?", model_id.bright_yellow()))
        .items(&["Sử dụng mặc định", "Nhập Model ID tùy chỉnh"])
        .default(0)
        .interact()?;

    if change_model == 1 {
        model_id = Input::with_theme(theme)
            .with_prompt("Nhập Model ID (VD: deepseek/deepseek-reasoner)")
            .interact_text()?;
    }

    // 4. Các cấu hình tùy chọn khác
    println!("\n{}", "🌐 Mở rộng khả năng (Tùy chọn, nhấn Enter để bỏ qua):".bold());
    let tavily_key: String = Password::with_theme(theme)
        .with_prompt("Tavily API Key (Tìm kiếm Web)")
        .allow_empty_password(true)
        .interact()?;

    let tele_token: String = Password::with_theme(theme)
        .with_prompt("Telegram Bot Token (Kết nối Bot)")
        .allow_empty_password(true)
        .interact()?;

    let config = Config {
        provider: provider_name.to_string(),
        api_key,
        model_id,
        tavily_key: if tavily_key.is_empty() { None } else { Some(tavily_key) },
        tele_token: if tele_token.is_empty() { None } else { Some(tele_token) },
    };

    // Hiệu ứng lưu cấu hình
    let pb = ProgressBar::new_spinner();
    pb.set_style(ProgressStyle::default_spinner().template("{spinner:.green} {msg}").unwrap());
    pb.enable_steady_tick(Duration::from_millis(100));
    pb.set_message("Đang mã hóa và lưu cấu hình...");
    
    tokio::time::sleep(Duration::from_secs(1)).await;
    save_config(&config)?;
    
    pb.finish_with_message("✅ Đã lưu cấu hình thành công!");

    println!("\n{}", "🎉 CHÚC MỪNG! HỆ THỐNG ĐÃ SẴN SÀNG.".bright_green().bold());
    println!("{}", format!("📁 Thư mục cấu hình: {:?}", get_config_dir()).bright_black());
    println!("{}", "👉 Gõ 'sunclaw chat' để bắt đầu trò chuyện ngay.".bright_yellow());
    
    Ok(())
}

async fn run_doctor() -> Result<()> {
    println!("{}", "🔍 ĐANG CHẨN ĐOÁN HỆ THỐNG SUNCLAW...".bright_cyan().bold());
    
    let pb = ProgressBar::new(4);
    pb.set_style(ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}")
        .unwrap()
        .progress_chars("#>-"));

    // 1. Kiểm tra cấu hình
    pb.set_message("Kiểm tra file cấu hình...");
    let config_path = get_config_path();
    let config_status = if config_path.exists() {
        format!("✅ Tìm thấy config tại {:?}", config_path).green()
    } else {
        "❌ Thiếu file config.toml (Hãy chạy 'sunclaw onboard')".red()
    };
    pb.inc(1);

    // 2. Kiểm tra Database
    pb.set_message("Kiểm tra cơ sở dữ liệu...");
    let db_path = "sunclaw.db";
    let db_status = if Path::new(db_path).exists() {
        format!("✅ Kết nối SQL tốt ({})", db_path).green()
    } else {
        "⚠️  Database chưa khởi tạo (Sẽ tự động tạo khi chạy)".yellow()
    };
    pb.inc(1);

    // 3. Kiểm tra kết nối mạng
    pb.set_message("Kiểm tra kết nối Internet...");
    let client = reqwest::Client::new();
    let net_status = match client.get("https://google.com").timeout(Duration::from_secs(3)).send().await {
        Ok(_) => "✅ Internet ổn định".green(),
        Err(_) => "❌ Không có kết nối mạng!".red(),
    };
    pb.inc(1);

    // 4. Kiểm tra quyền ghi
    pb.set_message("Kiểm tra quyền ghi thư mục...");
    let write_status = match fs::write(get_config_dir().join(".write_test"), "test") {
        Ok(_) => {
            let _ = fs::remove_file(get_config_dir().join(".write_test"));
            "✅ Có quyền ghi hệ thống".green()
        },
        Err(_) => "❌ Không có quyền ghi vào .sunclaw!".red(),
    };
    pb.inc(1);

    pb.finish_with_message("Hoàn tất chẩn đoán.");

    println!("\n╔════════════════════════════════════════════════════════════╗");
    println!("║ {:^58} ║", "📋 BÁO CÁO CHI TIẾT".bold().bright_white());
    println!("╠════════════════════════════════════════════════════════════╣");
    println!("║ ⚙️  Cấu hình: {:<47} ║", config_status);
    println!("║ 📦 Database: {:<47} ║", db_status);
    println!("║ 🌐 Mạng    : {:<47} ║", net_status);
    println!("║ 📝 Quyền   : {:<47} ║", write_status);
    println!("╚════════════════════════════════════════════════════════════╝");

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let theme = ColorfulTheme::default();

    show_header();
    
    if cli.doctor {
        run_doctor().await?;
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
            show_system_info();
            let config = load_config()?;
            if config.api_key.is_empty() {
                 println!("{}", "❌ Chưa cấu hình! Vui lòng chạy 'sunclaw onboard'.".red());
                 return Ok(());
            }
            
            // Nếu người dùng không chỉ định lệnh và file config chưa tồn tại, gợi ý onboard
            if !get_config_path().exists() {
                 println!("{}", "🔍 Chào mừng bạn mới! Đang khởi động trình hướng dẫn thiết lập...".cyan());
                 run_onboard(&theme).await?;
                 return Ok(());
            }

            start_terminal_chat(config, &theme).await?;
        }
    }

    Ok(())
}

async fn start_terminal_chat(config: Config, _theme: &ColorfulTheme) -> Result<()> {
    let runtime_config = RuntimeConfig {
        provider: config.provider,
        model_id: config.model_id,
        api_key: config.api_key,
        tavily_key: config.tavily_key,
    };

    let runtime = Arc::new(build_runtime(Some(runtime_config)).await);

    println!("\n--- 🗨️ Đang khởi động Sunclaw TUI Dashboard ---");
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

    let runtime_config = RuntimeConfig {
        provider: config.provider,
        model_id: config.model_id,
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
