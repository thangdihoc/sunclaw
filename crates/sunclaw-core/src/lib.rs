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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuditEvent {
    pub trace_id: String,
    pub skill: Option<String>,
    pub tool_name: String,
    pub decision: AuditDecision,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuditDecision {
    Allowed,
    Denied(String),
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
    async fn decide(&self, ctx: &AgentContext, messages: &[Message])
        -> Result<Decision, CoreError>;
}

#[async_trait]
pub trait Tool: Send + Sync {
    fn name(&self) -> &'static str;
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
}

#[async_trait]
pub trait MemoryStore: Send + Sync {
    async fn load_messages(&self, trace_id: &str) -> Result<Vec<Message>, CoreError>;
    async fn append_message(&self, trace_id: &str, message: Message) -> Result<(), CoreError>;
}
