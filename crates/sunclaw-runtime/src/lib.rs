use std::collections::HashMap;
use std::sync::Arc;

use sunclaw_core::{
    AgentContext, CoreError, Decision, MemoryStore, Message, ModelProvider, PolicyEngine, Role,
    Tool,
};

pub struct Runtime {
    provider: Arc<dyn ModelProvider>,
    memory: Arc<dyn MemoryStore>,
    policy: Arc<dyn PolicyEngine>,
    tools: HashMap<String, Arc<dyn Tool>>,
}

impl Runtime {
    pub fn new(
        provider: Arc<dyn ModelProvider>,
        memory: Arc<dyn MemoryStore>,
        policy: Arc<dyn PolicyEngine>,
    ) -> Self {
        Self {
            provider,
            memory,
            policy,
            tools: HashMap::new(),
        }
    }

    pub fn register_tool(&mut self, tool: Arc<dyn Tool>) {
        self.tools.insert(tool.name().to_string(), tool);
    }

    pub async fn run_once(
        &self,
        ctx: &AgentContext,
        user_input: &str,
    ) -> Result<String, CoreError> {
        self.memory
            .append_message(
                &ctx.trace_id,
                Message {
                    role: Role::User,
                    content: user_input.to_string(),
                },
            )
            .await?;

        let messages = self.memory.load_messages(&ctx.trace_id).await?;
        match self.provider.decide(ctx, &messages).await? {
            Decision::Reply(text) => {
                self.memory
                    .append_message(
                        &ctx.trace_id,
                        Message {
                            role: Role::Agent,
                            content: text.clone(),
                        },
                    )
                    .await?;
                Ok(text)
            }
            Decision::UseTool(call) => {
                self.policy.can_call_tool(ctx, &call.name).await?;
                let tool = self
                    .tools
                    .get(&call.name)
                    .ok_or_else(|| CoreError::Tool(format!("unknown tool: {}", call.name)))?;
                let result = tool.run(&call.input).await?;
                self.memory
                    .append_message(
                        &ctx.trace_id,
                        Message {
                            role: Role::Agent,
                            content: result.output.clone(),
                        },
                    )
                    .await?;
                Ok(result.output)
            }
        }
    }
}
