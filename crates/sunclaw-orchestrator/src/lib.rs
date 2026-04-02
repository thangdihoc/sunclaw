use serde::{Deserialize, Serialize};
use sunclaw_core::AgentRole;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HandoffStep {
    pub role: AgentRole,
    pub instructions: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TeamFlow {
    pub name: String,
    pub steps: Vec<HandoffStep>,
}

impl TeamFlow {
    pub fn planner_executor_reviewer() -> Self {
        Self {
            name: "planner-executor-reviewer".to_string(),
            steps: vec![
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
            ],
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.steps.is_empty() {
            return Err("team flow must include at least one step".to_string());
        }

        if self.steps.first().map(|s| &s.role) != Some(&AgentRole::Planner) {
            return Err("team flow must start with Planner".to_string());
        }

        if self.steps.last().map(|s| &s.role) != Some(&AgentRole::Reviewer) {
            return Err("team flow must end with Reviewer".to_string());
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::TeamFlow;

    #[test]
    fn default_flow_is_valid() {
        let flow = TeamFlow::planner_executor_reviewer();
        assert!(flow.validate().is_ok());
    }
}
