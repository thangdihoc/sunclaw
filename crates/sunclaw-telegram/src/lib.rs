use std::sync::Arc;
use teloxide::prelude::*;
use teloxide::types::ChatId;
use sunclaw_runtime::Runtime;
use sunclaw_core::AgentContext;
use anyhow::Result;

pub struct TelegramBridge {
    runtime: Arc<Runtime>,
    authorized_chat_id: i64,
}

impl TelegramBridge {
    pub fn new(runtime: Arc<Runtime>, authorized_chat_id: i64) -> Self {
        Self {
            runtime,
            authorized_chat_id,
        }
    }

    pub async fn run(&self, token: String) -> Result<()> {
        let bot = Bot::new(token);
        let runtime = self.runtime.clone();
        let authorized_chat_id = self.authorized_chat_id;

        println!("Sunclaw Telegram Bot is starting for chat_id: {}...", authorized_chat_id);

        teloxide::repl(bot, move |bot: Bot, msg: Message| {
            let runtime = runtime.clone();
            let authorized_chat_id = authorized_chat_id;
            
            async move {
                // Security Check: Only respond to authorized chat ID
                if msg.chat.id.0 != authorized_chat_id {
                    // Ignore or send a polite decline if it's the first time
                    return Ok(());
                }

                if let Some(text) = msg.text() {
                    let _ = bot.send_chat_action(msg.chat.id, teloxide::types::ChatAction::Typing).await;

                    let ctx = AgentContext {
                        trace_id: format!("tg-{}", msg.id.0),
                        skill: None,
                        model_profile: None,
                        role: None,
                        max_tokens: None,
                    };

                    match runtime.run_once(&ctx, text).await {
                        Ok(outcome) => {
                            bot.send_message(msg.chat.id, outcome.output).await?;
                        }
                        Err(e) => {
                            bot.send_message(msg.chat.id, format!("⚠️ Lỗi hệ thống: {:?}", e)).await?;
                        }
                    }
                }
                Ok(())
            }
        })
        .await;

        Ok(())
    }
}
