use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use sunclaw_core::{
    AgentContext, AuditEvent, AuditStore, CoreError, Decision, MemoryStore, Message, ModelProvider,
    Tool, ToolCall, ToolResult,
};
use sunclaw_policy::AllowlistPolicy;
use sunclaw_provider::{ModelRoute, MultiProvider, OpenAIProvider};
use sunclaw_runtime::{Runtime, RuntimeOptions};
use sunclaw_tools::WebSearchTool;

pub fn build_runtime() -> Runtime {
    let _ = dotenvy::dotenv();
    let openrouter_key = std::env::var("OPENROUTER_API_KEY").unwrap_or_else(|_| "secret".to_string());
    let tavily_key = std::env::var("TAVILY_API_KEY").unwrap_or_else(|_| "secret".to_string());

    let mut provider = MultiProvider::new("default");
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

    provider.add_backend(
        "openrouter:deepseek-v3",
        Arc::new(OpenAIProvider::new(
            &openrouter_key,
            "deepseek/deepseek-chat",
            None,
        )),
    );
    provider.add_backend(
        "xai:grok-3-mini",
        Arc::new(OpenAIProvider::new(
            &openrouter_key,
            "xai/grok-3-mini",
            None,
        )),
    );
    provider.add_backend(
        "openrouter:deepseek-r1",
        Arc::new(OpenAIProvider::new(
            &openrouter_key,
            "deepseek/deepseek-r1",
            None,
        )),
    );
    provider.add_backend(
        "google:gemini-2.5-pro",
        Arc::new(MockBackend::new("google:gemini-2.5-pro")),
    );
    provider.add_backend(
        "google:gemini-2.5-flash",
        Arc::new(MockBackend::new("google:gemini-2.5-flash")),
    );

    let memory = Arc::new(InMemoryStore::default());
    let policy = Arc::new(AllowlistPolicy::new(vec!["echo".into(), "web_search".into()]));
    let audit = Arc::new(InMemoryAuditStore::default());

    let mut runtime =
        Runtime::new(Arc::new(provider), memory, policy, audit).with_options(RuntimeOptions {
            max_turns: 4,
            max_tool_calls: 2,
        });
    runtime.register_tool(Arc::new(EchoTool));
    runtime.register_tool(Arc::new(WebSearchTool::new(tavily_key)));
    runtime
}

struct MockBackend {
    backend_id: &'static str,
}

impl MockBackend {
    fn new(backend_id: &'static str) -> Self {
        Self { backend_id }
    }
}

#[async_trait]
impl ModelProvider for MockBackend {
    async fn decide(
        &self,
        _ctx: &AgentContext,
        messages: &[Message],
    ) -> Result<Decision, CoreError> {
        let last = messages
            .last()
            .map(|m| m.content.clone())
            .unwrap_or_else(|| "hello".to_string());

        if last.contains("force_provider_fail") {
            return Err(CoreError::Provider(format!(
                "simulated provider error on {}",
                self.backend_id
            )));
        }

        if let Some(input) = last.strip_prefix("tool:") {
            return Ok(Decision::UseTool(ToolCall {
                name: "echo".into(),
                input: input.trim().to_string(),
            }));
        }

        if last.starts_with("tool:echo =>") {
            return Ok(Decision::Reply(format!(
                "[{0}] Tool completed: {last}",
                self.backend_id
            )));
        }

        Ok(Decision::Reply(format!(
            "[{0}] Sunclaw received: {last}",
            self.backend_id
        )))
    }
}

#[derive(Default)]
struct InMemoryStore {
    data: Mutex<HashMap<String, Vec<Message>>>,
}

#[async_trait]
impl MemoryStore for InMemoryStore {
    async fn load_messages(&self, trace_id: &str) -> Result<Vec<Message>, CoreError> {
        let guard = self
            .data
            .lock()
            .map_err(|e| CoreError::Memory(format!("lock error: {e}")))?;
        Ok(guard.get(trace_id).cloned().unwrap_or_default())
    }

    async fn append_message(&self, trace_id: &str, message: Message) -> Result<(), CoreError> {
        let mut guard = self
            .data
            .lock()
            .map_err(|e| CoreError::Memory(format!("lock error: {e}")))?;
        guard.entry(trace_id.to_string()).or_default().push(message);
        Ok(())
    }
}

#[derive(Default)]
struct InMemoryAuditStore {
    events: Mutex<Vec<AuditEvent>>,
}

#[async_trait]
impl AuditStore for InMemoryAuditStore {
    async fn append_event(&self, event: AuditEvent) -> Result<(), CoreError> {
        self.events
            .lock()
            .map_err(|e| CoreError::Memory(format!("lock error: {e}")))?
            .push(event);
        Ok(())
    }
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
