use serde::{Deserialize, Serialize};
use sunclaw_core::AgentRole;

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
