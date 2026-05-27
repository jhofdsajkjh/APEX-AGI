//! OMEGA AGI Supremacy — CLI binary entry point.
//!
//! Loads configuration, initialises all five layers + the Agent, and
//! runs user-requested tasks through the ReAct loop.
//!
//! Usage:
//!   omega-agi run "<task description>"    Run an AGI agent task
//!   omega-agi check                        Run health check & diagnostics
//!   omega-agi version                      Show version info

use omega_agent::{llm::LLMClient, tools::ToolContext, tools::ToolResult, Agent};
use omega_evolution::auto_evolve::{AutoEvolve, AutoEvolveConfig};
use std::sync::Arc;

// ---------------------------------------------------------------------------
// ToolContext implementation — bridges the Agent into the 5 layers
// ---------------------------------------------------------------------------

struct OmegaContext {
    omega: omega_agi::OmegaAGI,
}

#[async_trait::async_trait]
impl ToolContext for OmegaContext {
    async fn execute_tool(&self, name: &str, args: &serde_json::Value) -> ToolResult {
        match name {
            // ── System Health ──────────────────────────────────────────
            "health" => {
                let snap = self.omega.hypercore.health.snapshot();
                let json = serde_json::to_string_pretty(&snap).unwrap_or_else(|e| e.to_string());
                ToolResult::ok("health", json)
            }

            // ── Diagnostics ────────────────────────────────────────────
            "diagnose" => {
                // Register standard subsystems for a meaningful report
                let report = {
                    let diag = &self.omega.hypercore.diagnostics;
                    serde_json::to_string_pretty(&diag.run_diagnostics())
                        .unwrap_or_else(|e| e.to_string())
                };
                ToolResult::ok("diagnose", report)
            }

            // ── Self-Heal ──────────────────────────────────────────────
            "heal" => {
                let target = args
                    .get("target")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown");
                let issue = args
                    .get("issue")
                    .and_then(|v| v.as_str())
                    .unwrap_or("no issue specified");
                let result = self.omega.hypercore.healing.try_heal_with_retry(target, issue);
                let output = format!(
                    "Healing result for '{}': success={}, message={}, duration={}ms",
                    target,
                    result.success,
                    result.message,
                    result.duration_ms,
                );
                ToolResult::ok("heal", output)
            }

            // ── Code Generation ────────────────────────────────────────
            "codegen" => {
                let description = args
                    .get("description")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown");
                let language = args
                    .get("language")
                    .and_then(|v| v.as_str())
                    .unwrap_or("rust");
                let lang = match language.to_lowercase().as_str() {
                    "python" | "py" => omega_engineering::Language::Python,
                    "both" => omega_engineering::Language::Both,
                    _ => omega_engineering::Language::Rust,
                };
                let generated = self
                    .omega
                    .engineering
                    .generator
                    .generate_code(description, lang);
                let output = format!(
                    "Generated {} code (confidence: {:.2}):\n```\n{}\n```",
                    generated.language, generated.confidence, generated.code
                );
                ToolResult::ok("codegen", output)
            }

            // ── Evolution ──────────────────────────────────────────────
            "evolve" => {
                let result = self.omega.evolution.evolve();
                let output = format!(
                    "Evolution cycle complete. success={}, final_score={:.4}, iterations={}, message={}",
                    result.success,
                    result.final_score,
                    result.iterations,
                    result.message,
                );
                ToolResult::ok("evolve", output)
            }

            // ── Full Evolution (Auto-Evolve Pipeline) ──────────────────
            "evolve_full" => {
                let mut auto = AutoEvolve::new(AutoEvolveConfig {
                    workspace_root: "C:\\Users\\Fogtao\\Downloads\\omega-agi-supremacy".into(),
                    ..AutoEvolveConfig::default()
                });
                let mut evolver = self.omega.evolution.lock_evolver();
                let result = auto.run_once(&mut evolver);
                let output = format!(
                    "Auto-evolve cycle complete.\n\
                     ├─ Evolution: success={}, final_score={:.4}, iterations={}\n\
                     ├─ Tests passed: {}\n\
                     ├─ Fix attempts: {}\n\
                     ├─ Files generated: {}\n\
                     ├─ Duration: {}ms\n\
                     └─ Commit: {}",
                    result.evolution.success,
                    result.evolution.final_score,
                    result.evolution.iterations,
                    result.all_passed,
                    result.fix_attempts,
                    result.generated_files.len(),
                    result.duration_ms,
                    result.commit_hash.as_deref().unwrap_or("none"),
                );
                ToolResult::ok("evolve_full", output)
            }

            // ── File Read ──────────────────────────────────────────────
            "read" => {
                let path = args.get("path").and_then(|v| v.as_str()).unwrap_or("");
                if path.is_empty() {
                    return ToolResult::err("read", "Missing 'path' argument");
                }
                match std::fs::read_to_string(path) {
                    Ok(content) => ToolResult::ok("read", content),
                    Err(e) => ToolResult::err("read", format!("Cannot read '{}': {}", path, e)),
                }
            }

            // ── File Write ─────────────────────────────────────────────
            "write" => {
                let path = args.get("path").and_then(|v| v.as_str()).unwrap_or("");
                let content = args.get("content").and_then(|v| v.as_str()).unwrap_or("");
                if path.is_empty() {
                    return ToolResult::err("write", "Missing 'path' argument");
                }
                // Ensure parent directory exists
                if let Some(parent) = std::path::Path::new(path).parent() {
                    let _ = std::fs::create_dir_all(parent);
                }
                match std::fs::write(path, content) {
                    Ok(_) => ToolResult::ok("write", format!("Wrote {} bytes to {}", content.len(), path)),
                    Err(e) => ToolResult::err("write", format!("Cannot write '{}': {}", path, e)),
                }
            }

            // ── Code Search ────────────────────────────────────────────
            "search" => {
                let pattern = args.get("pattern").and_then(|v| v.as_str()).unwrap_or("");
                if pattern.is_empty() {
                    return ToolResult::err("search", "Missing 'pattern' argument");
                }
                let project_dir = "C:\\Users\\Fogtao\\Downloads\\omega-agi-supremacy";
                let mut results = Vec::new();
                if let Ok(entries) = walk_dir(project_dir, pattern) {
                    results = entries;
                }
                if results.is_empty() {
                    ToolResult::ok("search", format!("No matches found for pattern: {}", pattern))
                } else {
                    ToolResult::ok("search", results.join("\n"))
                }
            }

            // ── List Directory ─────────────────────────────────────────
            "ls" => {
                let path = args.get("path").and_then(|v| v.as_str()).unwrap_or(".");
                match std::fs::read_dir(path) {
                    Ok(entries) => {
                        let mut lines = Vec::new();
                        for entry in entries.flatten() {
                            let name = entry.file_name().to_string_lossy().to_string();
                            let kind = if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                                "DIR"
                            } else {
                                "FILE"
                            };
                            lines.push(format!("[{}] {}", kind, name));
                        }
                        ToolResult::ok("ls", lines.join("\n"))
                    }
                    Err(e) => ToolResult::err("ls", format!("Cannot list '{}': {}", path, e)),
                }
            }

            // ── Shell Command ──────────────────────────────────────────
            "bash" => {
                let command = args.get("command").and_then(|v| v.as_str()).unwrap_or("");
                if command.is_empty() {
                    return ToolResult::err("bash", "Missing 'command' argument");
                }
                match std::process::Command::new("cmd")
                    .args(["/C", command])
                    .output()
                {
                    Ok(output) => {
                        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                        let mut result = String::new();
                        if !stdout.is_empty() {
                            result.push_str(&stdout);
                        }
                        if !stderr.is_empty() {
                            if !result.is_empty() {
                                result.push('\n');
                            }
                            result.push_str(&format!("[stderr]\n{}", stderr));
                        }
                        if result.is_empty() {
                            result = "Command completed (no output)".to_string();
                        }
                        ToolResult::ok("bash", result)
                    }
                    Err(e) => ToolResult::err("bash", format!("Command failed: {}", e)),
                }
            }

            other => ToolResult::err(name, format!("Unknown tool: '{}'", other)),
        }
    }
}

/// Simple recursive directory walker for the `search` tool.
fn walk_dir(root: &str, pattern: &str) -> std::io::Result<Vec<String>> {
    let mut results = Vec::new();
    let lower_pattern = pattern.to_lowercase();
    walk_dir_inner(std::path::Path::new(root), &lower_pattern, &mut results, 0)?;
    Ok(results)
}

fn walk_dir_inner(
    dir: &std::path::Path,
    pattern: &str,
    results: &mut Vec<String>,
    depth: usize,
) -> std::io::Result<()> {
    if depth > 8 {
        return Ok(());
    }
    if dir.is_dir() {
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                let name = path.file_name().unwrap_or_default().to_string_lossy();
                if matches!(name.as_ref(), "node_modules" | ".git" | "target" | "__pycache__") {
                    continue;
                }
                walk_dir_inner(&path, pattern, results, depth + 1)?;
            } else if let Ok(content) = std::fs::read_to_string(&path) {
                if content.to_lowercase().contains(pattern) {
                    results.push(path.to_string_lossy().to_string());
                }
            }
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// CLI entry point
// ---------------------------------------------------------------------------

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load .env from project root
    let env_path = std::path::Path::new("C:\\Users\\Fogtao\\Downloads\\omega-agi-supremacy\\.env");
    if env_path.exists() {
        dotenvy::from_path(env_path).ok();
    } else {
        let example = env_path.with_extension("env.example");
        if example.exists() {
            dotenvy::from_path(&example).ok();
        }
    }

    // Initialise tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "omega_agi=info,omega_agent=info".into()),
        )
        .init();

    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        eprintln!("OMEGA AGI Supremacy v{}", env!("CARGO_PKG_VERSION"));
        eprintln!();
        eprintln!("Usage:");
        eprintln!("  omega-agi run \"<task description>\"    Run an AGI agent task");
        eprintln!("  omega-agi check                        Run health check & diagnostics");
        eprintln!("  omega-agi version                      Show version");
        return Ok(());
    }

    match args[1].as_str() {
        "run" => {
            let task = args[2..].join(" ");
            if task.is_empty() {
                anyhow::bail!("Please provide a task description. Usage: omega-agi run \"your task\"");
            }
            cmd_run(&task).await?;
        }
        "check" => {
            cmd_check().await?;
        }
        "version" => {
            println!("OMEGA AGI Supremacy v{}", env!("CARGO_PKG_VERSION"));
            println!("Layers: 5 (HyperCore, Runtime, Engineering, Evolution, Adapters)");
            println!("Agent: omega-agent enabled");
        }
        other => {
            anyhow::bail!("Unknown command '{}'. Use: run, check, version", other);
        }
    }

    Ok(())
}

/// `omega-agi run "<task>"` — create OmegaAGI + Agent and execute the task.
async fn cmd_run(task: &str) -> anyhow::Result<()> {
    tracing::info!("Initialising OMEGA AGI system...");

    let config = omega_agi::Config::new()
        .with_log_level(&std::env::var("LOG_LEVEL").unwrap_or_else(|_| "info".into()));
    let omega = omega_agi::OmegaAGI::new(config)?;

    tracing::info!(
        "System ready: {} layers, HyperCore v{}, Runtime v{}",
        omega.layer_count(),
        omega_hypercore::VERSION,
        omega_runtime::VERSION,
    );

    let context = Arc::new(OmegaContext { omega });

    let llm_client = LLMClient::from_env();
    if std::env::var("OMEGA_API_KEY").unwrap_or_default().is_empty() {
        tracing::warn!("OMEGA_API_KEY is not set — Agent will use mock responses. Set OMEGA_API_KEY in .env for real LLM access.");
    }
    let agent = Agent::new(llm_client, context);

    tracing::info!("Starting agent task...");
    let answer = agent.run(task).await?;

    println!("\n{}", "=".repeat(60));
    println!("🤖 OMEGA AGI Answer:");
    println!("{}", "=".repeat(60));
    println!("{}", answer);
    println!("{}", "=".repeat(60));

    Ok(())
}

/// `omega-agi check` — health check + diagnostics, no LLM needed.
async fn cmd_check() -> anyhow::Result<()> {
    let config = omega_agi::Config::new().with_log_level("info");
    let omega = omega_agi::OmegaAGI::new(config)?;

    println!("\n{}", "=".repeat(60));
    println!("🔍 OMEGA AGI System Health Check");
    println!("{}", "=".repeat(60));

    // Health snapshot
    let health = omega.hypercore.health.snapshot();
    println!("\n📊 Overall Health: {}", if health.overall_healthy { "✅ HEALTHY" } else { "❌ UNHEALTHY" });
    println!("   Uptime: {}s", health.uptime_seconds);
    println!("   Subsystems: {}", health.subsystems.len());

    for sub in &health.subsystems {
        let icon = if sub.healthy { "✅" } else { "❌" };
        println!("   {} {} — {} (errors: {})", icon, sub.name, sub.message, sub.error_count);
    }

    // Version info
    println!("\n📦 Version Info:");
    println!("   omega-agi:        v{}", env!("CARGO_PKG_VERSION"));
    println!("   omega-hypercore:  v{}", omega_hypercore::VERSION);
    println!("   omega-runtime:    v{}", omega_runtime::VERSION);
    println!("   omega-engineering: v{}", omega_engineering::VERSION);
    println!("   omega-evolution:  v{}", omega_evolution::VERSION);

    // Evolution stats
    let metrics = omega.evolution.evolver.lock().unwrap().get_metrics().clone();
    println!("\n🧬 Evolution Engine:");
    println!("   Iterations: {}", metrics.iterations);
    println!("   Best score: {:.4}", metrics.best_score);
    println!("   Current score: {:.4}", metrics.current_score);

    // LLM config check
    let api_key = std::env::var("OMEGA_API_KEY").unwrap_or_default();
    let model = std::env::var("OMEGA_MODEL_NAME").unwrap_or_else(|_| "gpt-4o".into());
    println!("\n🤖 LLM Configuration:");
    println!("   Provider: {}", std::env::var("OMEGA_LLM_PROVIDER").unwrap_or_else(|_| "not set".into()));
    println!("   Model: {}", model);
    println!("   API Key: {}", if api_key.is_empty() { "⚠️  NOT SET" } else { "✅ Configured" });

    println!("\n{}", "=".repeat(60));
    println!("✅ Check complete.");
    println!("{}", "=".repeat(60));

    Ok(())
}
