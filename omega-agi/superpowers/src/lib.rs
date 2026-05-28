//! # OMEGA AGI - Superpowers Engine (Layer 8)
//!
//! System enhancement layer providing:
//! - **Optimizer**: auto-tune system parameters for peak performance
//! - **Boost**: performance boost modes (Eco / Normal / Turbo / Overdrive)
//! - **SelfHeal**: automatic system repair and issue resolution
//! - **Analyzer**: deep system analysis and recommendations
//!
//! ## Architecture
//!
//! ```text
//! Superpowers
//! ├── Optimizer  (auto-parameter tuning)
//! ├── Boost      (performance modes)
//! ├── SelfHeal   (auto-repair engine)
//! └── Analyzer   (system analysis + recs)
//! ```

pub mod analyzer;
pub mod boost;
pub mod optimizer;
pub mod self_heal;

use analyzer::{AnalysisReport, Analyzer};
pub use boost::{BoostManager, BoostMode};
use optimizer::Optimizer;
use self_heal::SelfHealEngine;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Superpowers configuration
#[derive(Debug, Clone)]
pub struct SuperpowerConfig {
    /// Enable auto-optimization
    pub auto_optimize: bool,
    /// Default boost mode
    pub default_boost: BoostMode,
    /// Self-heal interval in seconds
    pub heal_interval: u64,
    /// Max auto-heal attempts per issue
    pub max_heal_attempts: u32,
    /// Enable deep analysis
    pub enable_analysis: bool,
    /// Analysis depth (1-5)
    pub analysis_depth: u32,
}

impl Default for SuperpowerConfig {
    fn default() -> Self {
        Self {
            auto_optimize: true,
            default_boost: BoostMode::Normal,
            heal_interval: 60,
            max_heal_attempts: 3,
            enable_analysis: true,
            analysis_depth: 3,
        }
    }
}

/// Enhanced system capability status
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SuperpowerStatus {
    pub active_boost: BoostMode,
    pub optimization_level: f64,
    pub last_optimization: Option<String>,
    pub issues_healed: u32,
    pub issues_pending: u32,
    pub system_score: f64,
    pub recommendations: Vec<String>,
}

/// The main Superpowers engine — enhances and amplifies system capabilities
#[allow(dead_code)]
pub struct Superpowers {
    config: SuperpowerConfig,
    optimizer: Arc<RwLock<Optimizer>>,
    boost: BoostManager,
    heal: SelfHealEngine,
    analyzer: Analyzer,
    issues_healed: Arc<RwLock<u32>>,
}

impl Superpowers {
    pub fn new() -> Self {
        Self::with_config(SuperpowerConfig::default())
    }

    pub fn with_config(config: SuperpowerConfig) -> Self {
        Self {
            optimizer: Arc::new(RwLock::new(Optimizer::new(config.auto_optimize))),
            boost: BoostManager::new(config.default_boost),
            heal: SelfHealEngine::new(config.max_heal_attempts),
            analyzer: Analyzer::new(config.analysis_depth, config.enable_analysis),
            issues_healed: Arc::new(RwLock::new(0)),
            config,
        }
    }

    /// Activate a performance boost mode
    pub fn activate_boost(&self, mode: BoostMode) -> String {
        let result = self.boost.activate(mode);
        tracing::info!(mode = ?mode, "Boost mode activated");
        result
    }

    /// Get current boost mode
    pub fn current_boost(&self) -> BoostMode {
        self.boost.current()
    }

    /// Run auto-optimization cycle
    pub async fn optimize(&self) -> String {
        let opt = self.optimizer.write().await;
        let result = opt.run().await;
        tracing::info!("Auto-optimization complete: {}", result);
        result
    }

    /// Get optimization level
    pub async fn optimization_level(&self) -> f64 {
        self.optimizer.read().await.level()
    }

    /// Run self-heal diagnostics and fixes
    pub async fn heal(&self) -> Vec<String> {
        let fixes = self.heal.run().await;
        let mut count = self.issues_healed.write().await;
        *count += fixes.len() as u32;
        tracing::info!("Self-heal applied {} fixes", fixes.len());
        fixes
    }

    /// Run full system analysis
    pub async fn analyze(&self) -> AnalysisReport {
        self.analyzer.analyze().await
    }

    /// Get comprehensive superpower status
    pub async fn status(&self) -> SuperpowerStatus {
        let healed = *self.issues_healed.read().await;
        let analysis = self.analyzer.analyze().await;
        let opt_level = self.optimizer.read().await.level();

        SuperpowerStatus {
            active_boost: self.boost.current(),
            optimization_level: opt_level,
            last_optimization: Some(chrono::Utc::now().to_rfc3339()),
            issues_healed: healed,
            issues_pending: analysis.issues_pending as u32,
            system_score: analysis.system_score,
            recommendations: analysis.recommendations,
        }
    }

    /// Version string
    pub fn version() -> &'static str {
        env!("CARGO_PKG_VERSION")
    }
}

impl Default for Superpowers {
    fn default() -> Self {
        Self::new()
    }
}
