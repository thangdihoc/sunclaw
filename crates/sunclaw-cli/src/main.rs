use anyhow::Result;
use sunclaw_app::build_runtime;
use sunclaw_core::AgentContext;

#[tokio::main]
async fn main() -> Result<()> {
    let runtime = build_runtime();
    let ctx = AgentContext {
        trace_id: "demo-trace".to_string(),
        skill: Some("general".to_string()),
    };

    let input = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "hello sunclaw".to_string());
    let outcome = runtime.run_once(&ctx, &input).await?;
    println!(
        "{}\n(turns={}, tool_calls={})",
        outcome.output, outcome.turns, outcome.tool_calls
    );
    Ok(())
}
