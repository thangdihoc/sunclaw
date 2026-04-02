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

pub async fn build_runtime() -> Runtime {
    let _ = dotenvy::dotenv();
    let openrouter_key = std::env::var("OPENROUTER_API_KEY").unwrap_or_else(|_| "secret".to_string());
    let tavily_key = std::env::var("TAVILY_API_KEY").unwrap_or_else(|_| "secret".to_string());

    let sqlite_store = Arc::new(
        SqliteStore::new("sqlite:sunclaw.db?mode=rwc")
            .await
            .expect("Failed to initialize SQLite store"),
    );

    let mut router = MultiProvider::new("default");
    provider.add_route(ModelRoute::new(
        "default",
        vec![
            "openrouter:deepseek-v3".to_string(),
            "xai:grok-3-mini".to_string(),
        ],
    ));
    provider.add_route(ModelRoute::new(
        "reasoning",
        vec![
            "openrouter:deepseek-r1".to_string(),
            "google:gemini-2.5-pro".to_string(),
        ],
    ));
    provider.add_route(ModelRoute::new(
        "cheap",
        vec![
            "google:gemini-2.5-flash".to_string(),
            "openrouter:deepseek-v3".to_string(),
        ],
    ));

    router.add_backend(
        "openrouter:deepseek-v3",
        Arc::new(RetryProvider::new(
            Arc::new(OpenAIProvider::new(
                &openrouter_key,
                "deepseek/deepseek-chat",
                None,
            )),
            3, // Max retries
        )),
    );
    router.add_backend(
        "xai:grok-3-mini",
        Arc::new(RetryProvider::new(
            Arc::new(OpenAIProvider::new(
                &openrouter_key,
                "xai/grok-3-mini",
                None,
            )),
            3,
        )),
    );
    router.add_backend(
        "openrouter:deepseek-r1",
        Arc::new(RetryProvider::new(
            Arc::new(OpenAIProvider::new(
                &openrouter_key,
                "deepseek/deepseek-r1",
                None,
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
