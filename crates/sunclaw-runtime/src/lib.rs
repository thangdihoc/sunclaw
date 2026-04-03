use std::collections::HashMap;
use std::sync::Arc;

use sunclaw_core::{
    AgentContext, AuditDecision, AuditEvent, AuditStore, CoreError, Decision, MemoryStore, Message,
    ModelProvider, PolicyEngine, Role, Tool,
};

#[derive(Debug, Clone)]
pub struct RuntimeOptions {
    pub max_turns: usize,
    pub max_tool_calls: usize,
    pub max_context_tokens: usize,
    pub tool_timeout: std::time::Duration,
}

impl Default for RuntimeOptions {
    fn default() -> Self {
        Self {
            max_turns: 4,
            max_tool_calls: 2,
            max_context_tokens: 4096,
            tool_timeout: std::time::Duration::from_secs(30),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeOutcome {
    pub output: String,
    pub turns: usize,
    pub tool_calls: usize,
}

pub struct Runtime {
    provider: Arc<dyn ModelProvider>,
    memory: Arc<dyn MemoryStore>,
    policy: Arc<dyn PolicyEngine>,
    audit: Arc<dyn AuditStore>,
    tools: HashMap<String, Arc<dyn Tool>>,
    options: RuntimeOptions,
}

pub struct ContextManager;

impl ContextManager {
    pub fn prune(messages: &[Message], max_tokens: usize) -> Vec<Message> {
        let mut system_msgs = Vec::new();
        let mut other_msgs = Vec::new();

        for msg in messages {
            if msg.role == Role::System {
                system_msgs.push(msg.clone());
            } else {
                other_msgs.push(msg.clone());
            }
        }

        let mut current_tokens = system_msgs.iter().map(|m| m.estimate_tokens()).sum::<usize>();
        let mut final_other_msgs = Vec::new();

        // Add non-system messages starting from the most recent
        for msg in other_msgs.into_iter().rev() {
            let msg_tokens = msg.estimate_tokens();
            if current_tokens + msg_tokens <= max_tokens {
                current_tokens += msg_tokens;
                final_other_msgs.push(msg);
            } else {
                break;
            }
        }
        
        // Reverse back to chronological order
        final_other_msgs.reverse();

        let mut result = system_msgs;
        result.extend(final_other_msgs);
        result
    }
}

impl Runtime {
    pub fn new(
        provider: Arc<dyn ModelProvider>,
        memory: Arc<dyn MemoryStore>,
        policy: Arc<dyn PolicyEngine>,
        audit: Arc<dyn AuditStore>,
    ) -> Self {
        Self {
            provider,
            memory,
            policy,
            audit,
            tools: HashMap::new(),
            options: RuntimeOptions::default(),
        }
    }

    pub fn with_options(mut self, options: RuntimeOptions) -> Self {
        self.options = options;
        self
    }

    pub fn register_tool(&mut self, tool: Arc<dyn Tool>) {
        self.tools.insert(tool.name().to_string(), tool);
    }


    pub async fn run_once(
        &self,
        ctx: &AgentContext,
        user_input: &str,
    ) -> Result<RuntimeOutcome, CoreError> {
        self.memory
            .append_message(
                &ctx.trace_id,
                Message {
                    role: Role::User,
                    content: user_input.to_string(),
                },
            )
            .await?;

        let mut turns = 0usize;
        let mut tool_calls = 0usize;

        while turns < self.options.max_turns {
            turns += 1;
            let all_messages = self.memory.load_messages(&ctx.trace_id).await?;
            
            // Context Management: Limit tokens
            let limit = ctx.max_tokens.unwrap_or(self.options.max_context_tokens);
            let pruned_messages = ContextManager::prune(&all_messages, limit);
            
            if pruned_messages.len() < all_messages.len() {
                println!("! [Runtime] Context pruned from {} to {} messages due to token limit ({})", 
                    all_messages.len(), pruned_messages.len(), limit);
            }

            // Collect tool definitions
            let tool_definitions: Vec<_> = self.tools.values().map(|t| t.definition()).collect();

            match self.provider.decide(ctx, &pruned_messages, &tool_definitions).await? {
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
                    return Ok(RuntimeOutcome {
                        output: text,
                        turns,
                        tool_calls,
                    });
                }
                Decision::UseTool(call) => {
                    if tool_calls >= self.options.max_tool_calls {
                        return Err(CoreError::Runtime(format!(
                            "tool limit reached: {}",
                            self.options.max_tool_calls
                        )));
                    }
                    tool_calls += 1;

                    if let Err(err) = self.policy.can_call_tool(ctx, &call.name, &call.input).await {
                        self.audit
                            .append_event(AuditEvent {
                                trace_id: ctx.trace_id.clone(),
                                skill: ctx.skill.clone(),
                                tool_name: call.name.clone(),
                                decision: AuditDecision::Denied(err.to_string()),
                            })
                            .await?;
                        return Err(err);
                    }

                    self.audit
                        .append_event(AuditEvent {
                            trace_id: ctx.trace_id.clone(),
                            skill: ctx.skill.clone(),
                            tool_name: call.name.clone(),
                            decision: AuditDecision::Allowed,
                        })
                        .await?;

                    let tool = self
                        .tools
                        .get(&call.name)
                        .ok_or_else(|| CoreError::Tool(format!("unknown tool: {}", call.name)))?;
                    
                    // Execute tool with timeout
                    let result = match tokio::time::timeout(self.options.tool_timeout, tool.run(&call.input)).await {
                        Ok(res) => res?,
                        Err(_) => {
                            return Err(CoreError::Tool(format!(
                                "tool execution timed out after {:?}",
                                self.options.tool_timeout
                            )))
                        }
                    };

                    self.memory
                        .append_message(
                            &ctx.trace_id,
                            Message {
                                role: Role::System,
                                content: format!("tool:{} => {}", call.name, result.output),
                            },
                        )
                        .await?;
                }
            }
        }

        Err(CoreError::Runtime(format!(
            "turn limit reached: {}",
            self.options.max_turns
        )))
    }
}

#[cfg(test)]
mod tests {
    use std::collections::{HashMap, VecDeque};
    use std::sync::{Arc, Mutex};

    use async_trait::async_trait;
    use sunclaw_core::{
        AgentContext, AuditEvent, AuditStore, CoreError, Decision, MemoryStore, Message,
        ModelProvider, PolicyEngine, Tool, ToolCall, ToolResult,
    };

    use crate::{Runtime, RuntimeOptions};

    struct StubProvider {
        decisions: Mutex<VecDeque<Decision>>,
    }

    #[async_trait]
    impl ModelProvider for StubProvider {
        async fn decide(
            &self,
            _ctx: &AgentContext,
            _messages: &[Message],
            _tools: &[sunclaw_core::ToolDefinition],
        ) -> Result<Decision, CoreError> {
            self.decisions
                .lock()
                .map_err(|e| CoreError::Provider(format!("lock error: {e}")))?
                .pop_front()
                .ok_or_else(|| CoreError::Provider("no decision queued".to_string()))
        }
    }

    struct AllowAll;

    #[async_trait]
    impl PolicyEngine for AllowAll {
        async fn can_call_tool(
            &self,
            _ctx: &AgentContext,
            _tool_name: &str,
            _tool_input: &str,
        ) -> Result<(), CoreError> {
            Ok(())
        }
    }

    #[derive(Default)]
    struct InMemory {
        messages: Mutex<HashMap<String, Vec<Message>>>,
    }

    #[async_trait]
    impl MemoryStore for InMemory {
        async fn load_messages(&self, trace_id: &str) -> Result<Vec<Message>, CoreError> {
            Ok(self
                .messages
                .lock()
                .map_err(|e| CoreError::Memory(format!("lock error: {e}")))?
                .get(trace_id)
                .cloned()
                .unwrap_or_default())
        }

        async fn append_message(&self, trace_id: &str, message: Message) -> Result<(), CoreError> {
            self.messages
                .lock()
                .map_err(|e| CoreError::Memory(format!("lock error: {e}")))?
                .entry(trace_id.to_string())
                .or_default()
                .push(message);
            Ok(())
        }
    }

    #[derive(Default)]
    struct InMemoryAudit {
        events: Mutex<Vec<AuditEvent>>,
    }

    #[async_trait]
    impl AuditStore for InMemoryAudit {
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

        fn definition(&self) -> sunclaw_core::ToolDefinition {
            sunclaw_core::ToolDefinition {
                name: "echo".to_string(),
                description: "Echo the input back".to_string(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "input": { "type": "string" }
                    }
                }),
            }
        }

        async fn run(&self, input: &str) -> Result<ToolResult, CoreError> {
            Ok(ToolResult {
                output: format!("echo:{input}"),
            })
        }
    }

    #[tokio::test]
    async fn runtime_executes_tool_then_returns_reply() {
        let provider = Arc::new(StubProvider {
            decisions: Mutex::new(VecDeque::from(vec![
                Decision::UseTool(ToolCall {
                    name: "echo".to_string(),
                    input: "hi".to_string(),
                }),
                Decision::Reply("done".to_string()),
            ])),
        });
        let memory = Arc::new(InMemory::default());
        let policy = Arc::new(AllowAll);
        let audit = Arc::new(InMemoryAudit::default());

        let mut runtime =
            Runtime::new(provider, memory, policy, audit).with_options(RuntimeOptions {
                max_turns: 3,
                max_tool_calls: 1,
                max_context_tokens: 1000,
                tool_timeout: std::time::Duration::from_secs(5),
            });
        runtime.register_tool(Arc::new(EchoTool));

        let out = runtime
            .run_once(
                &AgentContext {
                    trace_id: "t-1".to_string(),
                    skill: Some("general".to_string()),
                    model_profile: Some("default".to_string()),
                    role: None,
                    max_tokens: None,
                },
                "hello",
            )
            .await
            .expect("run should succeed");

        assert_eq!(out.output, "done");
        assert_eq!(out.tool_calls, 1);
    }
}
