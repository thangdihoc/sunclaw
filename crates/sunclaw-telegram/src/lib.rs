use std::sync::Arc;
use teloxide::prelude::*;
use async_trait::async_trait;
use sunclaw_core::{Bridge, CoreError, AgentContext};
// use sunclaw_app::build_runtime;
use sunclaw_runtime::Runtime;

pub struct TelegramBridge {
    runtime: Arc<Runtime>,
    token: String,
    authorized_chat_id: Option<i64>,
}

impl TelegramBridge {
    pub fn new(runtime: Arc<Runtime>, token: String, authorized_chat_id: Option<i64>) -> Self {
        Self {
            runtime,
            token,
            authorized_chat_id,
        }
    }
}

#[async_trait]
impl Bridge for TelegramBridge {
    fn name(&self) -> &'static str {
        "telegram"
    }

    async fn start(&self) -> Result<(), CoreError> {
        let bot = Bot::new(&self.token);
        let runtime = self.runtime.clone();
        let auth_id = self.authorized_chat_id;

        println!("! [Telegram] Starting bot...");

        teloxide::repl(bot, move |bot: Bot, msg: Message| {
            let runtime = runtime.clone();
            async move {
                let chat_id = msg.chat.id;
                
                // Security check
                if let Some(authorized) = auth_id {
                    if chat_id.0 != authorized {
                        println!("! [Telegram] Unauthorized access attempt from {}", chat_id);
                        bot.send_message(chat_id, "🚫 Bạn không có quyền truy cập Bot này.").await?;
                        return Ok(());
                    }
                }

                let user_text = msg.text().unwrap_or("");
                if user_text.is_empty() {
                    return Ok(());
                }

                // Gửi trạng thái đang gõ (Typing...)
                let _ = bot.send_chat_action(chat_id, teloxide::types::ChatAction::Typing).await;

                let trace_id = format!("tg-{}", chat_id);
                let ctx = AgentContext {
                    trace_id,
                    skill: Some("telegram".to_string()),
                    model_profile: Some("default".to_string()),
                    role: None,
                    max_tokens: None,
                };

                match runtime.run_once(&ctx, user_text).await {
                    Ok(outcome) => {
                        bot.send_message(chat_id, format!("*🤖 Sunclaw AI*\n\n{}", outcome.output))
                            .parse_mode(teloxide::types::ParseMode::MarkdownV2)
                            .await?;
                    }
                    Err(e) => {
                        bot.send_message(chat_id, format!("❌ *Lỗi:* `{}`", e))
                            .parse_mode(teloxide::types::ParseMode::MarkdownV2)
                            .await?;
                    }
                }
                Ok(())
            }
        })
        .await;

        Ok(())
    }
}
