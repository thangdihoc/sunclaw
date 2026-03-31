use std::collections::{HashMap, HashSet};

use async_trait::async_trait;
use sunclaw_core::{AgentContext, CoreError, PolicyEngine};

pub struct AllowlistPolicy {
    global_allowed_tools: HashSet<String>,
    skill_allowed_tools: HashMap<String, HashSet<String>>,
}

impl AllowlistPolicy {
    pub fn new(allowed_tools: Vec<String>) -> Self {
        Self {
            global_allowed_tools: allowed_tools.into_iter().collect(),
            skill_allowed_tools: HashMap::new(),
        }
    }

    pub fn with_skill_rules(mut self, skill: &str, allowed_tools: Vec<String>) -> Self {
        self.skill_allowed_tools
            .insert(skill.to_string(), allowed_tools.into_iter().collect());
        self
    }
}

#[async_trait]
impl PolicyEngine for AllowlistPolicy {
    async fn can_call_tool(&self, ctx: &AgentContext, tool_name: &str) -> Result<(), CoreError> {
        if !self.global_allowed_tools.contains(tool_name) {
            return Err(CoreError::PolicyDenied(format!(
                "tool not allowed by global policy: {tool_name}"
            )));
        }

        if let Some(skill) = &ctx.skill {
            if let Some(allowed) = self.skill_allowed_tools.get(skill) {
                if !allowed.contains(tool_name) {
                    return Err(CoreError::PolicyDenied(format!(
                        "tool not allowed for skill '{skill}': {tool_name}"
                    )));
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use sunclaw_core::{AgentContext, PolicyEngine};

    use crate::AllowlistPolicy;

    #[tokio::test]
    async fn denies_tool_not_in_skill_profile() {
        let policy = AllowlistPolicy::new(vec!["echo".to_string(), "search".to_string()])
            .with_skill_rules("general", vec!["echo".to_string()]);
        let result = policy
            .can_call_tool(
                &AgentContext {
                    trace_id: "x".to_string(),
                    skill: Some("general".to_string()),
                },
                "search",
            )
            .await;

        assert!(result.is_err());
    }
}
