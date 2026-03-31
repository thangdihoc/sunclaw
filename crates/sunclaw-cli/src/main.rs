use anyhow::Result;
use sunclaw_app::build_runtime;
use sunclaw_core::AgentContext;

#[tokio::main]
async fn main() -> Result<()> {
    let runtime = build_runtime();

    let mut args = std::env::args().skip(1);
    let mut model_profile = "default".to_string();
    let mut input = "hello sunclaw".to_string();

    while let Some(arg) = args.next() {
        if arg == "--profile" {
            if let Some(value) = args.next() {
                model_profile = value;
            }
            continue;
        }
        input = arg;
        break;
    }

    let ctx = AgentContext {
        trace_id: "demo-trace".to_string(),
        skill: Some("general".to_string()),
        model_profile: Some(model_profile.clone()),
    };

    let outcome = runtime.run_once(&ctx, &input).await?;
    println!(
        "profile={}\n{}\n(turns={}, tool_calls={})",
        model_profile, outcome.output, outcome.turns, outcome.tool_calls
    );
    Ok(())
}
