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
    Manager,
    Planner,
    Researcher,
    Coder,
    Writer,
    Reviewer,
    Custom(String),
}

impl AgentRole {
    pub fn get_system_instructions(&self) -> String {
        match self {
            AgentRole::Manager => "Bạn là Quản lý (Manager). Nhiệm vụ của bạn là điều phối toàn bộ dự án, ra quyết định cuối cùng và đảm bảo tiến độ.".to_string(),
            AgentRole::Planner => "Bạn là Người Lập Kế Hoạch (Planner). Hãy chia nhỏ yêu cầu người dùng thành các bước thực hiện cụ thể.".to_string(),
            AgentRole::Researcher => "Bạn là Nhà Nghiên Cứu (Researcher). Hãy tìm kiếm và tổng hợp thông tin chính xác từ các nguồn tin cậy.".to_string(),
            AgentRole::Coder => "Bạn là Lập Trình Viên (Coder). Hãy viết mã nguồn tối ưu, sạch sẽ và giải quyết các bài toán kỹ thuật.".to_string(),
            AgentRole::Writer => "Bạn là Người Viết Lách (Writer). Hãy biên soạn nội dung, báo cáo hoặc tài liệu một cách chuyên nghiệp.".to_string(),
            AgentRole::Reviewer => "Bạn là Người Đánh Giá (Reviewer). Hãy kiểm tra kết quả của các Agent khác để đảm bảo chất lượng.".to_string(),
            AgentRole::Custom(s) => format!("Bạn đang đảm nhận vai trò: {}. Hãy thực hiện nhiệm vụ theo đúng mô tả vai trò.", s),
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
    pub artifacts: Vec<Artifact>,
}

impl ToolResult {
    pub fn simple(output: &str) -> Self {
        Self {
            output: output.to_string(),
            artifacts: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Decision {
    Reply(String),
    UseTool(ToolCall),
}

#[derive(Debug, Clone, Default)]
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
pub enum MissionStatus {
    Pending,
    InProgress,
    Completed,
    Failed(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMetadata {
    pub id: String,
    pub name: String,
    pub role: AgentRole,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mission {
    pub id: String,
    pub title: String,
    pub description: String,
    pub status: MissionStatus,
    pub assign_to: Option<AgentRole>,
    pub sub_tasks: Vec<String>, // IDs of sub-missions
    pub parent_id: Option<String>,
    pub trace_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Artifact {
    pub id: String,
    pub trace_id: String,
    pub artifact_type: String, // 'file', 'chart', 'json', 'report'
    pub title: String,
    pub data: String, // Trình bày dưới dạng markdown hoặc base64
}

#[async_trait]
pub trait ArtifactStore: Send + Sync {
    async fn create_artifact(&self, artifact: Artifact) -> Result<(), CoreError>;
    async fn get_artifact(&self, id: &str) -> Result<Artifact, CoreError>;
    async fn list_artifacts_by_trace(&self, trace_id: &str) -> Result<Vec<Artifact>, CoreError>;
}

#[async_trait]
pub trait MissionStore: Send + Sync {
    async fn create_mission(&self, mission: Mission) -> Result<(), CoreError>;
    async fn update_mission_status(&self, id: &str, status: MissionStatus) -> Result<(), CoreError>;
    async fn get_mission(&self, id: &str) -> Result<Mission, CoreError>;
    async fn list_missions(&self) -> Result<Vec<Mission>, CoreError>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceEvent {
    pub trace_id: String,
    pub event_type: String, 
    pub content: String,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
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

#[async_trait]
pub trait TraceStore: Send + Sync {
    async fn append_trace(&self, event: TraceEvent) -> Result<(), CoreError>;
    async fn load_traces(&self, trace_id: &str) -> Result<Vec<TraceEvent>, CoreError>;
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
#[macro_export]
macro_rules! sunclaw_tool {
    ($struct_name:ident, $args_struct:ident, $name:expr, $description:expr, $self_name:ident, $args_name:ident, $body:block) => {
        #[async_trait]
        impl sunclaw_core::Tool for $struct_name {
            fn name(&self) -> &'static str {
                $name
            }

            fn definition(&self) -> sunclaw_core::ToolDefinition {
                sunclaw_core::ToolDefinition {
                    name: $name.to_string(),
                    description: $description.to_string(),
                    parameters: serde_json::to_value(
                        schemars::schema_for!($args_struct)
                    ).unwrap_or(serde_json::json!({})),
                }
            }

            async fn run(&self, input: &str) -> Result<sunclaw_core::ToolResult, sunclaw_core::CoreError> {
                let $self_name = self;
                let $args_name: $args_struct = serde_json::from_str(input)
                    .map_err(|e| sunclaw_core::CoreError::Tool(format!("Invalid input: {}", e)))?;
                $body
            }
        }
    };
}
