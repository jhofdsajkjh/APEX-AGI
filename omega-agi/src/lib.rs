//! # OMEGA AGI Supremacy
//!
//! Ten-layer AGI system: HyperCore → Runtime → Engineering → Evolution → Adapters → Agent → Research → Life-Harness → Superpowers → Avatar → Transcendence.
//!
//! ## Layers
//! - **Layer 0** `omega_hypercore` — Zero-allocation async runtime, persistent memory, capability security
//! - **Layer 1** `omega_runtime` — Actor system, WASM sandbox, effect system, ML inference, graph executor
//! - **Layer 2** `omega_engineering` — Code generation, test harness, quality gates, PR automation
//! - **Layer 3** `omega_evolution` — Self-evolution engine, competitive analysis, cross-project learning
//! - **Layer 4** `omega_adapters` — OpenClaw / Hermes / OpenHuman / Feishu protocol adapters
//! - **Layer 5** `omega_agent` — ReAct agent with LLM integration and tool system
//! - **Layer 6** `omega_research` — Autonomous research engine with web search and report generation
//! - **Layer 7** `omega_life_harness` — System self-maintenance, heartbeat, auto-recovery
//! - **Layer 8** `omega_superpowers` — Auto-optimization, performance boost, self-healing
//! - **Layer 9** `omega_avatar` — Local human-like AI avatar with TUI interface
//! - **Layer 10** `omega_transcendence` — Self-aware meta-cognition, emergent capability discovery

pub use omega_adapters as adapters;
pub use omega_agent as agent;
pub use omega_avatar as avatar;
pub use omega_engineering as engineering;
pub use omega_evolution as evolution;
pub use omega_hypercore as hypercore;
pub use omega_life_harness as life_harness;
pub use omega_research as research;
pub use omega_runtime as runtime;
pub use omega_superpowers as superpowers;
pub use omega_transcendence as transcendence;

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, mpsc, RwLock};

/// Cross-layer event types for the event bus
#[derive(Debug, Clone)]
pub enum CrossLayerEvent {
    /// Layer health changed
    HealthUpdate { layer: String, score: f64 },
    /// Evolution cycle completed
    EvolutionCompleted { iteration: u64, fitness: f64 },
    /// Research completed
    ResearchCompleted { topic: String, sources: usize },
    /// Resource warning from LifeHarness
    ResourceWarning { cpu: f64, memory: f64, disk: f64 },
    /// System needs healing
    HealingRequired { target: String, issue: String },
    /// Agent completed a task
    TaskCompleted {
        agent: String,
        task: String,
        success: bool,
    },
    /// Transcendence state change
    TranscendenceUpdate { awareness: f64, phase: String },
    /// Graceful shutdown requested
    ShutdownRequested,
}

/// Event bus for cross-layer communication
#[derive(Clone)]
pub struct EventBus {
    tx: broadcast::Sender<CrossLayerEvent>,
    health_tx: mpsc::UnboundedSender<(String, f64)>,
    health_rx: Arc<RwLock<Option<mpsc::UnboundedReceiver<(String, f64)>>>>,
}

impl EventBus {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(256);
        let (health_tx, health_rx) = mpsc::unbounded_channel();
        Self {
            tx,
            health_tx,
            health_rx: Arc::new(RwLock::new(Some(health_rx))),
        }
    }

    /// Publish an event to all subscribers
    pub fn publish(&self, event: CrossLayerEvent) {
        let _ = self.tx.send(event);
    }

    /// Subscribe to all events
    pub fn subscribe(&self) -> broadcast::Receiver<CrossLayerEvent> {
        self.tx.subscribe()
    }

    /// Report layer health score
    pub fn report_health(&self, layer: &str, score: f64) {
        let _ = self.health_tx.send((layer.to_string(), score));
    }

    /// Drain health updates (caller must hold exclusive access)
    pub async fn drain_health(&self) -> HashMap<String, f64> {
        let mut rx_opt = self.health_rx.write().await;
        if let Some(rx) = rx_opt.as_mut() {
            let mut health = HashMap::new();
            while let Ok((layer, score)) = rx.try_recv() {
                health.insert(layer, score);
            }
            health
        } else {
            HashMap::new()
        }
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}

/// Top-level configuration with .env support
#[derive(Debug, Clone)]
pub struct Config {
    pub github_token: Option<String>,
    pub openai_api_key: Option<String>,
    pub log_level: String,
    pub data_dir: String,
    pub enable_transcendence: bool,
}

impl Config {
    pub fn new() -> Self {
        // Auto-load .env if present
        let _ = dotenvy::dotenv();

        Self {
            github_token: std::env::var("GITHUB_TOKEN").ok().filter(|s| !s.is_empty()),
            openai_api_key: std::env::var("OPENAI_API_KEY")
                .or_else(|_| std::env::var("OMEGA_API_KEY"))
                .ok()
                .filter(|s| !s.is_empty()),
            log_level: std::env::var("OMEGA_LOG_LEVEL").unwrap_or_else(|_| "info".into()),
            data_dir: std::env::var("OMEGA_DATA_DIR").unwrap_or_else(|_| "./data".into()),
            enable_transcendence: std::env::var("OMEGA_TRANSCENDENCE")
                .ok()
                .map_or(true, |v| v != "0" && v != "false"),
        }
    }

    pub fn with_github_token(mut self, token: &str) -> Self {
        self.github_token = Some(token.to_string());
        self
    }

    pub fn with_log_level(mut self, level: &str) -> Self {
        self.log_level = level.to_string();
        self
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

/// Layer health snapshot for cross-layer awareness
#[derive(Debug, Clone)]
pub struct LayerHealthSnapshot {
    pub layers: HashMap<String, f64>,
    pub timestamp: String,
}

/// Main OMEGA AGI entry point — all 11 layers with event bus
pub struct OmegaAGI {
    pub config: Config,
    pub event_bus: EventBus,
    pub hypercore: omega_hypercore::HyperCore,
    pub runtime: omega_runtime::Runtime,
    pub engineering: omega_engineering::Engineering,
    pub evolution: omega_evolution::Evolution,
    pub adapters: omega_adapters::AdapterManager,
    pub research: omega_research::Researcher,
    pub life_harness: omega_life_harness::LifeHarness,
    pub superpowers: omega_superpowers::Superpowers,
    pub avatar: omega_avatar::AvatarEngine,
    pub transcendence: omega_transcendence::TranscendenceEngine,
    layer_health: RwLock<HashMap<String, f64>>,
}

impl OmegaAGI {
    pub fn new(config: Config) -> anyhow::Result<Self> {
        let _ = tracing_subscriber::fmt()
            .with_env_filter(
                tracing_subscriber::EnvFilter::try_from_default_env()
                    .unwrap_or_else(|_| format!("omega_agi={}", config.log_level).into()),
            )
            .try_init();

        let event_bus = EventBus::new();

        let mut health = HashMap::new();
        health.insert("hypercore".into(), 1.0);
        health.insert("runtime".into(), 1.0);
        health.insert("engineering".into(), 0.9);
        health.insert("evolution".into(), 0.8);
        health.insert("adapters".into(), 0.9);
        health.insert("agent".into(), 0.7);
        health.insert("research".into(), 0.8);
        health.insert("life_harness".into(), 0.9);
        health.insert("superpowers".into(), 0.8);
        health.insert("avatar".into(), 0.9);
        health.insert(
            "transcendence".into(),
            if config.enable_transcendence {
                0.5
            } else {
                0.0
            },
        );

        Ok(Self {
            hypercore: omega_hypercore::HyperCore::new()?,
            runtime: omega_runtime::Runtime::new()?,
            engineering: omega_engineering::Engineering::new(),
            evolution: omega_evolution::Evolution::new(),
            adapters: omega_adapters::AdapterManager::new(),
            research: omega_research::Researcher::new(),
            life_harness: omega_life_harness::LifeHarness::new(),
            superpowers: omega_superpowers::Superpowers::new(),
            avatar: omega_avatar::AvatarEngine::new(),
            transcendence: omega_transcendence::TranscendenceEngine::new(),
            layer_health: RwLock::new(health),
            config,
            event_bus,
        })
    }

    /// Update a layer's health score and publish event
    pub async fn update_layer_health(&self, layer: &str, score: f64) {
        let mut health = self.layer_health.write().await;
        health.insert(layer.to_string(), score.clamp(0.0, 1.0));
        self.event_bus.report_health(layer, score);
        self.event_bus.publish(CrossLayerEvent::HealthUpdate {
            layer: layer.to_string(),
            score,
        });
    }

    /// Get current layer health snapshot
    pub async fn layer_health(&self) -> LayerHealthSnapshot {
        let health = self.layer_health.read().await;
        LayerHealthSnapshot {
            layers: health.clone(),
            timestamp: chrono::Utc::now().to_rfc3339(),
        }
    }

    /// Run transcendence meta-cognition cycle (Layer 10)
    pub async fn run_transcendence_cycle(&self) -> anyhow::Result<()> {
        if !self.config.enable_transcendence {
            return Ok(());
        }
        let health = self.layer_health.read().await;
        let state = self.transcendence.meta_cognition(health.clone()).await;
        let _discovered = self.transcendence.discover_emergent(&health).await;
        let _optimizations = self.transcendence.quantum_optimize(&health).await;
        let _goals = self
            .transcendence
            .self_actualize(state.awareness_level)
            .await;

        self.event_bus
            .publish(CrossLayerEvent::TranscendenceUpdate {
                awareness: state.awareness_level,
                phase: state.phase.clone(),
            });

        tracing::info!(
            awareness = %state.awareness_level,
            phase = %state.phase,
            "Transcendence cycle completed"
        );
        Ok(())
    }

    /// Full cross-layer health check
    pub async fn health_check(&self) -> anyhow::Result<String> {
        let health = self.layer_health.read().await;
        let mut output = String::new();
        output.push_str("┌─ OMEGA AGI System Health ──────────────────────┐\n");
        let mut sorted_layers: Vec<_> = health.iter().collect();
        sorted_layers.sort_by_key(|(k, _)| *k);
        for (layer, score) in &sorted_layers {
            let bar = match score {
                s if **s >= 0.9 => "🟢",
                s if **s >= 0.6 => "🟡",
                _ => "🔴",
            };
            output.push_str(&format!(
                "│  {} {:<14} {:>6.1}%\n",
                bar,
                layer,
                *score * 100.0
            ));
        }
        output.push_str("└────────────────────────────────────────────────┘\n");

        if self.config.enable_transcendence {
            let summary = self.transcendence.summary().await;
            output.push_str(&format!(
                "\n🧠 Transcendence: {} (awareness: {:.1}%, phase: {})\n  Capabilities: {} | Self-goals: {} | Optimizations: {}\n",
                if summary.awareness_level > 0.6 { "✨ ACTIVE" } else { "💤 awakening" },
                summary.awareness_level * 100.0,
                summary.phase,
                summary.emergent_capabilities.len(),
                summary.self_goals.len(),
                summary.total_optimizations,
            ));
        }

        Ok(output)
    }

    pub fn version(&self) -> &str {
        env!("CARGO_PKG_VERSION")
    }

    pub fn layer_count(&self) -> usize {
        11
    }
}

/// Build an agent tool registry with all system capabilities
pub fn build_system_tool_registry(omega: &OmegaAGI) -> agent::tool::ToolRegistry {
    use agent::tool::{FnTool, Tool};

    let mut tools = agent::tool::ToolRegistry::new();

    // Add built-in tools via FnTool wrapper
    tools.register(Box::new(FnTool::new(
        "think",
        "Use this tool for internal reasoning and planning",
        serde_json::json!({"thought": {"type": "string", "description": "Your reasoning"}}),
        std::sync::Arc::new(|_args| agent::tool::ToolResult::ok("think", "Thought recorded")),
    )));

    tools.register(Box::new(FnTool::new(
        "check_health",
        "Check the health status of all OMEGA AGI layers",
        serde_json::json!({}),
        std::sync::Arc::new(|_args| {
            agent::tool::ToolResult::ok(
                "check_health",
                "Use the 'health' CLI command for full details",
            )
        }),
    )));

    tools.register(Box::new(FnTool::new(
        "system_status",
        "Get detailed system resource usage (CPU, memory, disk, network)",
        serde_json::json!({}),
        std::sync::Arc::new(|_args| {
            agent::tool::ToolResult::ok(
                "system_status",
                "Use the 'life resources' CLI command for details",
            )
        }),
    )));

    tools.register(Box::new(FnTool::new(
        "list_adapters",
        "List all available protocol adapters (OpenClaw, Hermes, OpenHuman, Feishu)",
        serde_json::json!({}),
        std::sync::Arc::new(|_args| {
            agent::tool::ToolResult::ok(
                "list_adapters",
                "Available adapters: OpenClaw, Hermes, OpenHuman, Feishu",
            )
        }),
    )));

    tools
}
