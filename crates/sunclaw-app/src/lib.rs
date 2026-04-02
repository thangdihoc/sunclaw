use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use sunclaw_core::{
    AgentContext, AuditEvent, AuditStore, CoreError, Decision, MemoryStore, Message, ModelProvider,
    Tool, ToolCall, ToolResult,
};
use sunclaw_policy::AllowlistPolicy;
use sunclaw_provider::{ModelRoute, MultiProvider, OpenAIProvider, RetryProvider};
use sunclaw_runtime::{Runtime, RuntimeOptions};
use sunclaw_tools::WebSearchTool;
use sunclaw_memory_sqlite::SqliteStore;

pub struct RuntimeConfig {
    pub provider: String, // "openrouter", "openai", "anthropic", "google"
    pub model_id: String,
    pub api_key: String,
    pub tavily_key: Option<String>,
}

pub async fn build_runtime(config: Option<RuntimeConfig>) -> Runtime {
    let _ = dotenvy::dotenv();
    
    let (provider_name, model_id, api_key, tavily_key) = if let Some(c) = config {
        (c.provider, c.model_id, c.api_key, c.tavily_key.unwrap_or_else(|| "secret".to_string()))
    } else {
        (
            "openrouter".to_string(),
            "deepseek/deepseek-chat".to_string(),
            std::env::var("OPENROUTER_API_KEY").unwrap_or_else(|_| "secret".to_string()),
            std::env::var("TAVILY_API_KEY").unwrap_or_else(|_| "secret".to_string()),
        )
    };

    let sqlite_store = Arc::new(
        SqliteStore::new("sqlite:sunclaw.db?mode=rwc")
            .await
            .expect("Failed to initialize SQLite store"),
    );

    let mut router = MultiProvider::new("default");
    router.add_route(ModelRoute::new(
        "default",
        vec![model_id.clone()],
    ));

    let endpoint = match provider_name.as_str() {
        "openai" => Some("https://api.openai.com/v1".to_string()),
        "anthropic" => Some("https://api.anthropic.com/v1".to_string()), // Will need AnthropicProvider later
        "google" => Some("https://generativelanguage.googleapis.com/v1beta".to_string()),
        _ => Some("https://openrouter.ai/api/v1".to_string()),
    };

    router.add_backend(
        &model_id,
        Arc::new(RetryProvider::new(
            Arc::new(OpenAIProvider::new(
                &api_key,
                &model_id,
                endpoint,
            )),
            3,
        )),
    );

    let memory = sqlite_store.clone();
    let policy = Arc::new(AllowlistPolicy::new(vec!["echo".into(), "web_search".into()]));
    let audit = sqlite_store.clone();

    let mut runtime =
        Runtime::new(Arc::new(router), memory, policy, audit).with_options(RuntimeOptions {
            max_turns: 4,
            max_tool_calls: 2,
            max_context_tokens: 4096,
            tool_timeout: std::time::Duration::from_secs(30),
        });
    runtime.register_tool(Arc::new(EchoTool));
    runtime.register_tool(Arc::new(WebSearchTool::new(tavily_key)));
    runtime
}

struct EchoTool;

#[async_trait]
impl Tool for EchoTool {
    fn name(&self) -> &'static str {
        "echo"
    }

    async fn run(&self, input: &str) -> Result<ToolResult, CoreError> {
        Ok(ToolResult {
            output: format!("[tool:echo] {input}"),
        })
    }
}
