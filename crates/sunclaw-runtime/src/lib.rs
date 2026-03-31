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
}

impl Default for RuntimeOptions {
    fn default() -> Self {
        Self {
            max_turns: 4,
            max_tool_calls: 2,
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

                    if let Err(err) = self.policy.can_call_tool(ctx, &call.name).await {
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
                    let result = tool.run(&call.input).await?;
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
            });
        runtime.register_tool(Arc::new(EchoTool));

        let out = runtime
            .run_once(
                &AgentContext {
                    trace_id: "t-1".to_string(),
                    skill: Some("general".to_string()),
                    model_profile: Some("default".to_string()),
                },
                "hello",
            )
            .await
            .expect("run should succeed");

        assert_eq!(out.output, "done");
        assert_eq!(out.tool_calls, 1);
    }
}
