//! # OMEGA AGI Supremacy
//!
//! Five-layer AGI system: HyperCore → Runtime → Engineering → Evolution → Pipeline.
//!
//! ## Layers
//! - **Layer 0** `omega_hypercore` — Zero-allocation async runtime, persistent memory, capability security
//! - **Layer 1** `omega_runtime` — Actor system, WASM sandbox, effect system, ML inference, graph executor
//! - **Layer 2** `omega_engineering` — Code generation, test harness, quality gates, PR automation
//! - **Layer 3** `omega_evolution` — Self-evolution engine, competitive analysis, cross-project learning
//! - **Layer 4** `omega_adapters` — OpenClaw / Hermes / OpenHuman / Feishu protocol adapters

pub use omega_hypercore as hypercore;
pub use omega_runtime as runtime;
pub use omega_engineering as engineering;
pub use omega_evolution as evolution;
pub use omega_adapters as adapters;

/// Top-level configuration
#[derive(Debug, Clone)]
pub struct Config {
    pub github_token: Option<String>,
    pub log_level: String,
    pub data_dir: String,
}

impl Config {
    pub fn new() -> Self {
        Self {
            github_token: None,
            log_level: "info".to_string(),
            data_dir: "./data".to_string(),
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

/// Main OMEGA AGI entry point
pub struct OmegaAGI {
    pub config: Config,
    pub hypercore: omega_hypercore::HyperCore,
    pub runtime: omega_runtime::Runtime,
    pub engineering: omega_engineering::Engineering,
    pub evolution: omega_evolution::Evolution,
    pub adapters: omega_adapters::AdapterManager,
}

impl OmegaAGI {
    pub fn new(config: Config) -> anyhow::Result<Self> {
        let _ = tracing_subscriber::fmt()
            .with_env_filter(
                tracing_subscriber::EnvFilter::try_from_default_env()
                    .unwrap_or_else(|_| format!("omega_agi={}", config.log_level).into()),
            )
            .try_init();

        Ok(Self {
            hypercore: omega_hypercore::HyperCore::new()?,
            runtime: omega_runtime::Runtime::new()?,
            engineering: omega_engineering::Engineering::new(),
            evolution: omega_evolution::Evolution::new(),
            adapters: omega_adapters::AdapterManager::new(),
            config,
        })
    }

    pub fn version(&self) -> &str {
        env!("CARGO_PKG_VERSION")
    }

    pub fn layer_count(&self) -> usize {
        5
    }
}
