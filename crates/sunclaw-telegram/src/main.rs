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
    use sunclaw_core::Bridge;
    let bridge = TelegramBridge::new(runtime, token, Some(authorized_chat_id));
    bridge.start().await.map_err(|e| anyhow::anyhow!(e))?;

    Ok(())
}
