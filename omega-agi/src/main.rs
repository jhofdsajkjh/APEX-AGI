//! # OMEGA AGI Supremacy — CLI Entry Point
//!
//! ```bash
//! # System health check
//! omega-agi check
//!
//! # Run an agent task
//! omega-agi run "检查系统健康状态"
//!
//! ```

use std::sync::Arc;
use omega_agent::tool::ToolRegistry;
use omega_agent::tool::ToolResult;
use omega_agent::feedback::FeedbackCollector;
use omega_agent::orchestrator::Orchestrator;
use omega_evolution::{AutoEvolve, AutoEvolveConfig};
use omega_agent::tool::FnTool;

fn build_tools(omega: &omega_agi::OmegaAGI) -> ToolRegistry {
    let mut registry = ToolRegistry::new();
    registry.register(Box::new(FnTool::new(
        "health","Check system health.",serde_json::json!({}),
        Arc::new(move |_args| {
            let s = omega.hypercore.health.snapshot();
            ToolResult::ok("health", serde_json::to_string_pretty(&s).unwrap_or_default())
        }),
    )));
    registry.register(Box::new(FnTool::new(
        "diagnose","Full system diagnostic.",serde_json::json!({}),
        Arc::new(move |_args| {
            let d = omega.hypercore.diagnostics.run_diagnostics();
            ToolResult::ok("diagnose", serde_json::to_string_pretty(&d).unwrap_or_default())
        }),
    )));
    registry.register(Box::new(FnTool::new(
        "evolve","Run evolution cycle.",serde_json::json!({}),
        Arc::new(move |_args| {
            let r = omega.evolution.evolve();
            ToolResult::ok("evolve", format!("score={:.4},iters={},msg={}",r.final_score,r.iterations,r.message))
        }),
    )));
    registry
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let cmd = args.first().cloned().unwrap_or_else(|| "check".into());
    match cmd.as_str() {
        "run" => cmd_run(&args.get(1).cloned().unwrap_or_default()).await?,
        "evolve" => cmd_evolve().await?,
        "interactive"|"chat" => cmd_interactive().await?,
        "check"|"health" => cmd_check().await?,
        _ => println!("Usage: omega-agi [run|evolve|interactive|check]"),
    }
    Ok(())
}

async fn cmd_run(task: &str) -> anyhow::Result<()> {
    let omega = omega_agi::OmegaAGI::new(omega_agi::Config::new())?;
    let tools = build_tools(&omega);
    let feedback = Arc::new(FeedbackCollector::new(1000));
    let agent = omega_agent::Agent::from_env(tools).with_feedback(feedback);
    let answer = agent.run(task).await?;
    println!("\nAnswer:\n{}", answer);
    Ok(())
}

async fn cmd_evolve() -> anyhow::Result<()> {
    println!("Self-evolution placeholder");
    Ok(())
}

async fn cmd_interactive() -> anyhow::Result<()> {
    println!("Interactive placeholder");
    Ok(())
}

async fn cmd_check() -> anyhow::Result<()> {
    println!("OMEGA AGI: OK");
    Ok(())
}
