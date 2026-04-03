use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tiktoken_rs::cl100k_base;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Role {
    User,
    Agent,
    System,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AgentRole {
    Planner,
    Executor,
    Reviewer,
}

impl AgentRole {
    pub fn get_system_instructions(&self) -> String {
        match self {
            AgentRole::Planner => {
                "Bạn là Người Lập Kế Hoạch (Planner). Nhiệm vụ của bạn là chia nhỏ yêu cầu của người dùng thành các bước thực hiện cụ thể. Đừng tự thực hiện, chỉ lập kế hoạch.".to_string()
            }
            AgentRole::Executor => {
                "Bạn là Người Thực Thi (Executor). Hãy thực hiện các bước trong kế hoạch bằng cách sử dụng các công cụ được cho phép. Tập trung vào độ chính xác của kết quả.".to_string()
            }
            AgentRole::Reviewer => {
                "Bạn là Người Đánh Giá (Reviewer). Hãy kiểm tra kết quả từ Người Thực Thi đối chiếu với kế hoạch ban đầu. Đảm bảo mọi thứ đều đúng yêu cầu và an toàn.".to_string()
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Message {
    pub role: Role,
    pub content: String,
}

impl Message {
    pub fn estimate_tokens(&self) -> usize {
        let bpe = cl100k_base().unwrap();
        // Base tokens for every message (role, content formatting)
        let mut tokens = 3; 
        tokens += bpe.encode_with_special_tokens(&self.content).len();
        tokens
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ToolCall {
    pub name: String,
    pub input: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ToolResult {
    pub output: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Decision {
    Reply(String),
    UseTool(ToolCall),
}

#[derive(Debug, Clone)]
pub struct AgentContext {
    pub trace_id: String,
    pub skill: Option<String>,
    pub model_profile: Option<String>,
    pub role: Option<AgentRole>,
    pub max_tokens: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AuditEvent {
    pub trace_id: String,
    pub skill: Option<String>,
    pub tool_name: String,
    pub decision: AuditDecision,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuditDecision {
    Allowed,
    Denied(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value, // JSON Schema
}

#[derive(Debug, thiserror::Error)]
pub enum CoreError {
    #[error("provider error: {0}")]
    Provider(String),
    #[error("policy denied: {0}")]
    PolicyDenied(String),
    #[error("tool error: {0}")]
    Tool(String),
    #[error("memory error: {0}")]
    Memory(String),
    #[error("runtime error: {0}")]
    Runtime(String),
}

#[async_trait]
pub trait ModelProvider: Send + Sync {
    async fn decide(
        &self,
        ctx: &AgentContext,
        messages: &[Message],
        tools: &[ToolDefinition],
    ) -> Result<Decision, CoreError>;
}

#[async_trait]
pub trait Tool: Send + Sync {
    fn name(&self) -> &'static str;
    fn definition(&self) -> ToolDefinition;
    async fn run(&self, input: &str) -> Result<ToolResult, CoreError>;
}

#[async_trait]
pub trait PolicyEngine: Send + Sync {
    async fn can_call_tool(
        &self,
        ctx: &AgentContext,
        tool_name: &str,
        tool_input: &str,
    ) -> Result<(), CoreError>;
}

#[async_trait]
pub trait AuditStore: Send + Sync {
    async fn append_event(&self, event: AuditEvent) -> Result<(), CoreError>;
    async fn load_events(&self, trace_id: &str) -> Result<Vec<AuditEvent>, CoreError>;
}

#[async_trait]
pub trait MemoryStore: Send + Sync {
    async fn load_messages(&self, trace_id: &str) -> Result<Vec<Message>, CoreError>;
    async fn append_message(&self, trace_id: &str, message: Message) -> Result<(), CoreError>;
    async fn list_traces(&self) -> Result<Vec<String>, CoreError>;
}

#[macro_export]
macro_rules! sunclaw_tool {
    ($struct_name:ident, $args_type:ty, $name:expr, $desc:expr, $self_name:ident, $args_name:ident, $exec:block) => {
        #[async_trait]
        impl $crate::Tool for $struct_name {
            fn name(&self) -> &'static str { $name }
            fn definition(&self) -> $crate::ToolDefinition {
                let schema = schemars::schema_for!($args_type);
                $crate::ToolDefinition {
                    name: $name.to_string(),
                    description: $desc.to_string(),
                    parameters: serde_json::to_value(schema).unwrap(),
                }
            }
            async fn run(&self, input: &str) -> Result<$crate::ToolResult, $crate::CoreError> {
                let $args_name: $args_type = serde_json::from_str(input)
                    .map_err(|e| $crate::CoreError::Tool(format!("Invalid arguments for tool {}: {}", $name, e)))?;
                let $self_name = self;
                $exec
            }
        }
    };
}
#[async_trait]
pub trait Bridge: Send + Sync {
    fn name(&self) -> &'static str;
    async fn start(&self) -> Result<(), CoreError>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeConfig {
    pub name: String,
    pub enabled: bool,
    pub settings: serde_json::Value,
}
