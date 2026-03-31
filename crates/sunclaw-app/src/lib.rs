use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use sunclaw_core::{
    AgentContext, AuditEvent, AuditStore, CoreError, Decision, MemoryStore, Message, ModelProvider,
    Tool, ToolCall, ToolResult,
};
use sunclaw_policy::AllowlistPolicy;
use sunclaw_runtime::{Runtime, RuntimeOptions};

pub fn build_runtime() -> Runtime {
    let provider = Arc::new(MockProvider);
    let memory = Arc::new(InMemoryStore::default());
    let policy = Arc::new(AllowlistPolicy::new(vec!["echo".into()]));
    let audit = Arc::new(InMemoryAuditStore::default());

    let mut runtime = Runtime::new(provider, memory, policy, audit).with_options(RuntimeOptions {
        max_turns: 4,
        max_tool_calls: 2,
    });
    runtime.register_tool(Arc::new(EchoTool));
    runtime
}

struct MockProvider;

#[async_trait]
impl ModelProvider for MockProvider {
    async fn decide(
        &self,
        _ctx: &AgentContext,
        messages: &[Message],
    ) -> Result<Decision, CoreError> {
        let last = messages
            .last()
            .map(|m| m.content.clone())
            .unwrap_or_else(|| "hello".to_string());

        if let Some(input) = last.strip_prefix("tool:") {
            return Ok(Decision::UseTool(ToolCall {
                name: "echo".into(),
                input: input.trim().to_string(),
            }));
        }

        if last.starts_with("tool:echo =>") {
            return Ok(Decision::Reply(format!("Tool completed: {last}")));
        }

        Ok(Decision::Reply(format!("Sunclaw received: {last}")))
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
