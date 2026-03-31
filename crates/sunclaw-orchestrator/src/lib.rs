use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentRole {
    Planner,
    Executor,
    Reviewer,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandoffStep {
    pub role: AgentRole,
    pub instructions: String,
}

pub fn default_team_flow() -> Vec<HandoffStep> {
    vec![
        HandoffStep {
            role: AgentRole::Planner,
            instructions: "Break down the task into executable steps".into(),
        },
        HandoffStep {
            role: AgentRole::Executor,
            instructions: "Execute steps using allowed tools".into(),
        },
        HandoffStep {
            role: AgentRole::Reviewer,
            instructions: "Review for quality and policy compliance".into(),
        },
    ]
}
