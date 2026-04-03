use std::sync::Arc;
use async_trait::async_trait;
use serde_json::{json, Value};
use sunclaw_core::{AgentContext, AgentRole, CoreError, Tool, ToolDefinition, ToolResult};
use sunclaw_runtime::{Runtime, RuntimeOutcome};

/// AgentMember wraps a Sunclaw Runtime and exposes it as a Tool.
/// This allows one Agent (the Supervisor) to call another Agent.
pub struct AgentMember {
    name: String,
    description: String,
    runtime: Arc<Runtime>,
    // Default context for this agent member
    context: AgentContext,
}

impl AgentMember {
    pub fn new(name: &str, description: &str, runtime: Arc<Runtime>, context: AgentContext) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            runtime,
            context,
        }
    }
}

#[async_trait]
impl Tool for AgentMember {
    fn name(&self) -> &'static str {
        // We use a Boxed string or leak it for the &'static str requirement of the trait
        // For simplicity in this framework, we might need to change the Tool trait 
        // to return String or handle this differently.
        // For now, let's just use a leaky string to satisfy the current trait.
        Box::leak(self.name.clone().into_boxed_str())
    }

    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: self.name.clone(),
            description: self.description.clone(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "task": {
                        "type": "string",
                        "description": format!("The specific task for the {} agent to perform.", self.name)
                    }
                },
                "required": ["task"]
            }),
        }
    }

    async fn run(&self, input: &str) -> Result<ToolResult, CoreError> {
        let args: Value = serde_json::from_str(input).map_err(|e| {
            CoreError::Tool(format!("Invalid arguments for agent member {}: {}", self.name, e))
        })?;
        
        let task = args["task"].as_str().ok_or_else(|| {
            CoreError::Tool("Missing 'task' parameter for agent call".to_string())
        })?;

        tracing::info!("Delegating task to agent {}: {}", self.name, task);

        // Run the member agent
        let outcome = self.runtime.run_once(&self.context, task).await?;

        Ok(ToolResult {
            output: outcome.output,
        })
    }
}

/// A step in a sequential multi-agent flow.
pub struct TeamStep {
    pub role: AgentRole,
}

/// A pre-defined flow for a team of agents.
pub struct TeamFlow {
    pub steps: Vec<TeamStep>,
}

/// Orchestrator manages a collection of agents and provides coordination patterns.
pub struct Orchestrator {
    supervisor_runtime: Arc<Runtime>,
}

impl Orchestrator {
    pub fn new(supervisor_runtime: Arc<Runtime>) -> Self {
        Self {
            supervisor_runtime,
        }
    }

    /// Process a high-level goal through the supervisor (Hierarchical pattern).
    /// The supervisor must have AgentMembers registered as tools in its runtime.
    pub async fn run_hierarchical(&self, ctx: &AgentContext, goal: &str) -> Result<RuntimeOutcome, CoreError> {
        let mut supervisor_context = ctx.clone();
        supervisor_context.role = Some(AgentRole::Planner);
        
        self.supervisor_runtime.run_once(&supervisor_context, goal).await
    }

    /// Run a sequential flow where each agent's output is the next agent's input.
    pub async fn run_sequential(
        &self,
        ctx: &AgentContext,
        initial_input: &str,
        flow: &TeamFlow,
    ) -> Result<Vec<RuntimeOutcome>, CoreError> {
        let mut outcomes = Vec::new();
        let mut current_input = initial_input.to_string();

        for (i, step) in flow.steps.iter().enumerate() {
            let mut step_ctx = ctx.clone();
            step_ctx.role = Some(step.role.clone());
            step_ctx.trace_id = format!("{}-step-{}", ctx.trace_id, i);

            let outcome = self.supervisor_runtime.run_once(&step_ctx, &current_input).await?;
            current_input = outcome.output.clone();
            outcomes.push(outcome);
        }

        Ok(outcomes)
    }
}
