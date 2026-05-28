//! # Omega Evolution Library
//! Layer 3 - Self-evolution engine for continuous performance improvement.

pub mod auto_evolve;
pub mod self_evolve;
pub mod apex_core;

pub use self_evolve::{
    SelfEvolver,
    EvolverConfig,
    EvolutionResult,
    EvolutionMetrics,
    ImprovementResult,
    PerformanceSnapshot,
};

pub use auto_evolve::AutoEvolve;
pub use auto_evolve::AutoEvolveConfig;
pub use auto_evolve::AutoEvolveResult;

pub use apex_core::{
    compute_apex, apex_fitness, apex_guidance, format_apex_state,
    ApexInput, ApexState, ApexGuidance,
};

/// Evolution engine version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

use tokio::sync::RwLock;

/// Top-level evolution module wrapper — async-safe (uses tokio::sync::RwLock).
pub struct Evolution {
    pub evolver: RwLock<SelfEvolver>,
    pub config: EvolverConfig,
}

impl Evolution {
    pub fn new() -> Self {
        let config = EvolverConfig::default();
        Self {
            evolver: RwLock::new(SelfEvolver::new(config.clone())),
            config,
        }
    }

    pub fn with_config(config: EvolverConfig) -> Self {
        Self {
            evolver: RwLock::new(SelfEvolver::new(config.clone())),
            config,
        }
    }

    /// Run one evolution cycle (async-safe, uses tokio RwLock internally).
    pub async fn evolve(&self) -> EvolutionResult {
        self.evolver.write().await.evolve()
    }

    /// Access the locked evolver for custom operations.
    pub async fn lock_evolver(&self) -> tokio::sync::RwLockWriteGuard<'_, SelfEvolver> {
        self.evolver.write().await
    }
}

impl Default for Evolution {
    fn default() -> Self {
        Self::new()
    }
}
