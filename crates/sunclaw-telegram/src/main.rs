use std::sync::Arc;
use sunclaw_telegram::TelegramBridge;
use sunclaw_app::build_runtime;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load .env file
    let _ = dotenvy::dotenv();

    // Initialize logging
    tracing_subscriber::fmt::init();

    let token = std::env::var("TELEGRAM_BOT_TOKEN")
        .expect("TELEGRAM_BOT_TOKEN must be set");
    
    let chat_id_str = std::env::var("TELEGRAM_AUTHORIZED_CHAT_ID")
        .expect("TELEGRAM_AUTHORIZED_CHAT_ID must be set");
    
    let authorized_chat_id: i64 = chat_id_str.parse()
        .expect("TELEGRAM_AUTHORIZED_CHAT_ID must be a valid i64");

    // Build the Sunclaw runtime
    let runtime = Arc::new(build_runtime(None).await);

    // Create and run the bridge
    let bridge = TelegramBridge::new(runtime, authorized_chat_id);
    bridge.run(token).await?;

    Ok(())
}
