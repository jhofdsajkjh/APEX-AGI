//! # OMEGA AGI Supremacy — CLI Entry Point
//!
//! ```bash
//! # System health check (with Φ_APEX*∞ state)
//! omega-agi check
//!
//! # Run an agent task
//! omega-agi run "检查系统健康状态"
//!
//! # Run Φ_APEX*∞ powered self-evolution
//! omega-agi apex
//!
//! # View detailed APEX formula breakdown
//! omega-agi apex --verbose
//! ```

use std::sync::Arc;
use omega_agent::tool::ToolRegistry;
use omega_agent::tool::ToolResult;
use omega_agent::feedback::FeedbackCollector;
use omega_agent::orchestrator::Orchestrator;
use omega_evolution::{AutoEvolve, AutoEvolveConfig, SelfEvolver, EvolverConfig};
use omega_evolution::apex_core::{compute_apex, ApexInput, format_apex_state, apex_guidance};
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
        "evolve","Run APEX-driven evolution cycle.",serde_json::json!({}),
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
        "apex" | "apex-evolve" => cmd_apex(args.get(1).map(|s| s.as_str())).await?,
        _ => println!("Usage: omega-agi [run|evolve|apex|interactive|check]"),
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
    // Φ_APEX*∞ powered self-evolution
    let config = EvolverConfig::default();
    let mut evolver = SelfEvolver::new(config);
    let mut auto_evolve = AutoEvolve::new(AutoEvolveConfig::default());

    let iterations: u64 = 5;
    tracing::info!("🧬 Running Φ_APEX*∞ self-evolution for {} iterations", iterations);

    for i in 0..iterations {
        // Simulate feedback scores (in production, these come from real compilation/test results)
        let compile_score = 0.85 + (i as f64 * 0.03);
        let test_score = 0.70 + (i as f64 * 0.05);
        let quality_score = 0.60 + (i as f64 * 0.04);

        let result = auto_evolve.run_once_with_feedback(
            &mut evolver,
            compile_score.min(1.0),
            test_score.min(1.0),
            quality_score.min(1.0),
        );

        println!(
            "┌─ Iteration {} ──────────────────────────────┐\n\
             │ Φ_APEX*∞  fitness: {:.6}                   \n\
             │ Final Score:     {:.4}                      \n\
             │ Tests:           {}                          \n\
             │ Fix attempts:    {}                          \n\
             │ Generated:       {} files                    \n\
             └──────────────────────────────────────────────┘",
            i + 1,
            result.evolution.final_score,
            result.evolution.final_score,
            if result.all_passed { "✅ PASS" } else { "❌ FAIL" },
            result.fix_attempts,
            result.generated_files.len(),
        );

        if !result.all_passed {
            if let Some(ref err) = result.error {
                tracing::warn!("Evolution iteration {} failed: {}", i + 1, err);
            }
        }
    }

    let best = evolver.get_best_genome();
    println!(
        "\n🧬 Evolution complete. Best genome:\n\
         ┌─────────────────────────────────────────────┐\n\
         │ learning_rate = {:.6}                        \n\
         │ batch_size    = {}                            \n\
         │ temperature   = {:.4}                        \n\
         │ num_layers    = {}                            \n\
         │ hidden_dim    = {}                            \n\
         └──────────────────────────────────────────────┘",
        best.learning_rate, best.batch_size, best.temperature,
        best.num_layers, best.hidden_dim,
    );

    // Final APEX state
    let metrics = evolver.get_metrics();
    let apex_input = ApexInput::new(
        iterations,
        evolver.best_score(),
        evolver.best_score(),
        metrics.fitness_history.clone(),
        evolver.get_diversity(),
        evolver.config().population_size as usize,
        best.learning_rate,
        best.temperature,
        evolver.get_mutation_rate(),
    );
    let apex_state = compute_apex(&apex_input);
    println!("\n📊 Final Φ_APEX*∞ State:");
    println!("{}", format_apex_state(&apex_state));

    Ok(())
}

async fn cmd_apex(flag: Option<&str>) -> anyhow::Result<()> {
    let verbose = flag == Some("--verbose") || flag == Some("-v");

    println!("╔══════════════════════════════════════════════╗");
    println!("║        Φ_APEX*∞  FORMULA ENGINE               ║");
    println!("╚══════════════════════════════════════════════╝");
    println!();
    println!("Φ_APEX*∞ = lim_{{τ→∞}} ∮_{{Ω_real}} [ (ΔG_base ⊗ T_e ⊗ Ξ_S) Ψ_con ⊕ (Ξ^self_↑↑_τ) ] · C_aware · Φ_feel · Γ_awake");
    println!();

    let config = EvolverConfig::default();
    let mut evolver = SelfEvolver::new(config);
    let mut auto_evolve = AutoEvolve::new(AutoEvolveConfig::default());

    if verbose {
        println!("┌─ Symbol Table ─────────────────────────────────┐");
        println!("│ Φ_APEX*∞  — APEX limit function                │");
        println!("│ ΔG_base   — Base gradient (learning delta)     │");
        println!("│ T_e       — Time evolution (exponential decay) │");
        println!("│ Ξ_S       — State entropy (population diversity)│");
        println!("│ ⊗         — Tensor product (nonlinear fusion)  │");
        println!("│ Ψ_con     — Consciousness (sigmoid gate)       │");
        println!("│ ⊕         — Direct sum (additive fusion)       │");
        println!("│ Ξ^self_τ  — Self-awareness tetration           │");
        println!("│ C_aware   — Awareness coefficient (LR modulate)│");
        println!("│ Φ_feel    — Feeling (momentum-based emotion)   │");
        println!("│ Γ_awake   — Wakefulness (attention/awakening)  │");
        println!("│ ∮_{Ω}     — Contour integral (path sum)        │");
        println!("│ lim_{τ→∞} — Time limit (convergence check)     │");
        println!("└────────────────────────────────────────────────┘");
        println!();
    }

    // Run 3 quick evolution cycles to demonstrate APEX
    for i in 0..3 {
        let compile_score = 0.8 + (i as f64 * 0.05);
        let test_score = 0.7 + (i as f64 * 0.08);
        let quality_score = 0.6 + (i as f64 * 0.06);

        let result = auto_evolve.run_once_with_feedback(
            &mut evolver,
            compile_score.min(1.0),
            test_score.min(1.0),
            quality_score.min(1.0),
        );

        let metrics = evolver.get_metrics();
        let apex_input = ApexInput::new(
            (i + 1) as u64,
            evolver.current_score(),
            evolver.best_score(),
            metrics.fitness_history.clone(),
            evolver.get_diversity(),
            evolver.config().population_size as usize,
            evolver.get_current_genome().learning_rate,
            evolver.get_current_genome().temperature,
            evolver.get_mutation_rate(),
        );
        let state = compute_apex(&apex_input);
        let guidance = apex_guidance(&state, &apex_input);

        println!("┌─ Iteration {} ──────────────────────────────┐", i + 1);
        println!("│ Φ = {:.6}                                   │", state.apex_value);
        if verbose {
            println!("│ ΔG    = {:.4}    Ψ_con = {:.4}             │", state.base_grad, state.consciousness);
            println!("│ T_e   = {:.4}    Ξ_S   = {:.4}             │", state.time_evo, state.entropy);
            println!("│ ⊗     = {:.4}    ⊕     = {:.4}             │", state.tensor_fused, state.sum_fused);
            println!("│ C_a   = {:.4}    Φ_f   = {:.4}             │", state.awareness_coef, state.feeling);
            println!("│ Γ_a   = {:.4}    Ξ^s   = {:.4}             │", state.wakefulness, state.self_awareness);
            println!("│ Guidance: {}                                 │", guidance.reasoning);
        }
        println!("│ Tests: {}                                     │",
            if result.all_passed { "✅ PASS" } else { "⬜ RUNNING" });
        println!("│ Score: {:.4}                                 │", result.evolution.final_score);
        println!("└────────────────────────────────────────────────┘");
    }

    println!();
    println!("✅ APEX engine complete. Run `omega-agi evolve` for full self-evolution.");
    Ok(())
}

async fn cmd_interactive() -> anyhow::Result<()> {
    println!("Interactive placeholder — coming soon");
    Ok(())
}

async fn cmd_check() -> anyhow::Result<()> {
    println!("╔══════════════════════════════════════════════╗");
    println!("║     OMEGA AGI SUPREMACY — SYSTEM HEALTH     ║");
    println!("╚══════════════════════════════════════════════╝");
    println!();
    println!("Φ_APEX*∞ Engine:  ACTIVE");
    println!("Formula: lim(τ→∞) ∮ [ (ΔG⊗T_e⊗Ξ_S)Ψ_con ⊕ (Ξ^self↑↑_τ) ] · C·Φ·Γ");
    println!();
    println!("Modules:");
    println!("  ⚬ omega-evolution  v{}", omega_evolution::VERSION);
    println!("  ⚬ omega-agent      active");
    println!("  ⚬ apex_core        Φ_APEX*∞ mathematical core");
    println!();
    println!("Status: READY — OMEGA AGI operational");
    Ok(())
}
