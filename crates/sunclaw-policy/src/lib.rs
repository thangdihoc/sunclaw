use async_trait::async_trait;
use sunclaw_core::{AgentContext, CoreError, PolicyEngine};

pub struct AllowlistPolicy {
    allowed_tools: Vec<String>,
}

impl AllowlistPolicy {
    pub fn new(allowed_tools: Vec<String>) -> Self {
        Self { allowed_tools }
    }
}

#[async_trait]
impl PolicyEngine for AllowlistPolicy {
    async fn can_call_tool(&self, _ctx: &AgentContext, tool_name: &str) -> Result<(), CoreError> {
        if self.allowed_tools.iter().any(|name| name == tool_name) {
            Ok(())
        } else {
            Err(CoreError::PolicyDenied(format!(
                "tool not allowed: {tool_name}"
            )))
        }
    }
}
