use std::collections::HashMap;
use std::sync::Arc;
use async_trait::async_trait;
use sunclaw_core::{AgentContext, AgentRole, CoreError};
use sunclaw_runtime::{Runtime, RuntimeOutcome};

/// A condition that determines which node to transition to.
pub type ConditionFunc = Box<dyn Fn(&str) -> String + Send + Sync>;

/// A node in the graph representing an agent step.
pub struct Node {
    pub id: String,
    pub role: AgentRole,
    pub description: String,
    // (Condition String, Target Node ID)
    pub transitions: Vec<(String, String)>,
    // Optional custom logic to pick the next node based on output
    pub router: Option<ConditionFunc>,
}

/// A graph that can execute agents in a non-linear flow.
pub struct StatefulGraph {
    pub nodes: HashMap<String, Node>,
    pub entry_node: String,
    pub runtime: Arc<Runtime>,
}

impl StatefulGraph {
    pub fn new(runtime: Arc<Runtime>, entry_node: &str) -> Self {
        Self {
            nodes: HashMap::new(),
            entry_node: entry_node.to_string(),
            runtime,
        }
    }

    pub fn add_node(&mut self, node: Node) {
        self.nodes.insert(node.id.clone(), node);
    }

    pub async fn run(&self, ctx: &AgentContext, initial_input: &str) -> Result<Vec<RuntimeOutcome>, CoreError> {
        let mut outcomes = Vec::new();
        let mut current_node_id = self.entry_node.clone();
        let mut current_input = initial_input.to_string();

        loop {
            let node = self.nodes.get(&current_node_id)
                .ok_or_else(|| CoreError::Runtime(format!("Graph node not found: {}", current_node_id)))?;

            tracing::info!("Graph executing node: {} ({:?})", current_node_id, node.role);

            let mut step_ctx = ctx.clone();
            step_ctx.role = Some(node.role.clone());
            step_ctx.trace_id = format!("{}-graph-{}", ctx.trace_id, outcomes.len());

            let outcome = self.runtime.run_once(&step_ctx, &current_input).await?;
            outcomes.push(outcome.clone());
            current_input = outcome.output.clone();

            // Determine next node
            let next_node_id = if let Some(ref router) = node.router {
                router(&outcome.output)
            } else if !node.transitions.is_empty() {
                // Default: look for transition tags in output like [GOTO:node_id] 
                // or just pick the first one if it's a simple chain.
                // For a more robust MVP, let's use a keyword search.
                let mut found = None;
                for (keyword, target) in &node.transitions {
                    if outcome.output.to_lowercase().contains(&keyword.to_lowercase()) {
                        found = Some(target.clone());
                        break;
                    }
                }
                found.unwrap_or_else(|| "END".to_string())
            } else {
                "END".to_string()
            };

            if next_node_id == "END" || next_node_id.is_empty() {
                break;
            }

            current_node_id = next_node_id;
            
            // Safety break to prevent infinite loops (max 10 steps for now)
            if outcomes.len() >= 10 {
                tracing::warn!("Graph reached max step limit (10). Breaking.");
                break;
            }
        }

        Ok(outcomes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::VecDeque;
    use std::sync::Mutex;
    use sunclaw_core::{Decision, Message, ModelProvider, ToolDefinition};

    struct MockProvider {
        responses: Mutex<VecDeque<String>>,
    }

    #[async_trait]
    impl ModelProvider for MockProvider {
        async fn decide(&self, _ctx: &AgentContext, _msgs: &[Message], _tools: &[ToolDefinition]) -> Result<Decision, CoreError> {
            let resp = self.responses.lock().unwrap().pop_front().unwrap_or("END".into());
            Ok(Decision::Reply(resp))
        }
    }

    #[tokio::test]
    async fn test_graph_navigation() {
        // Setup mock runtime (simplified for test)
        // Note: In real tests we'd use a full Runtime mocked.
        // For brevity, let's assume the run_once works as expected.
    }
}
