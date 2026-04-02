use anyhow::Result;
use sunclaw_app::build_runtime;
use sunclaw_core::AgentContext;

#[tokio::main]
async fn main() -> Result<()> {
    let runtime = build_runtime().await;

    let args = std::env::args().skip(1).collect::<Vec<String>>();
    let mut model_profile = "default".to_string();
    let mut input = "hello sunclaw".to_string();
    let mut is_team = false;

    let mut i = 0;
    while i < args.len() {
        let arg = &args[i];
        if arg == "--profile" {
            if i + 1 < args.len() {
                model_profile = args[i + 1].clone();
                i += 2;
                continue;
            }
        } else if arg == "--team" {
            is_team = true;
            i += 1;
            continue;
        }
        input = arg.clone();
        break;
    }

    let ctx = AgentContext {
        trace_id: "demo-trace".to_string(),
        skill: Some("general".to_string()),
        model_profile: Some(model_profile.clone()),
        role: None,
    };

    if is_team {
        use sunclaw_orchestrator::TeamFlow;
        let flow = TeamFlow::planner_executor_reviewer();
        let outcomes = runtime.run_team_flow(&ctx, &input, &flow).await?;
        for (idx, outcome) in outcomes.iter().enumerate() {
            let role_name = match idx {
                0 => "Planner",
                1 => "Executor",
                2 => "Reviewer",
                _ => "Agent",
            };
            println!(
                "\n--- [{role_name}] ---\n{}\n(turns={}, tool_calls={})",
                outcome.output, outcome.turns, outcome.tool_calls
            );
        }
    } else {
        let outcome = runtime.run_once(&ctx, &input).await?;
        println!(
            "profile={}\n{}\n(turns={}, tool_calls={})",
            model_profile, outcome.output, outcome.turns, outcome.tool_calls
        );
    }
    Ok(())
}
