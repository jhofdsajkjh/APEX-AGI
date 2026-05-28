//! # Omega Evolution Library
//! Layer 3 - Self-evolution engine for continuous performance improvement.

// Nightly toolchain lints — allow pedantic format/naming warnings not applicable on stable
#![allow(
    clippy::too_many_arguments,
    clippy::unnecessary_cast,
    clippy::iter_cloned_collect,
    clippy::needless_bool
)]
#![allow(
    named_arguments_used_positionally,
    unused_mut,
    unused_imports,
    unused_variables,
    dead_code
)]

pub mod apex_core;
pub mod auto_evolve;
pub mod self_evolve;

pub use self_evolve::{
    EvolutionMetrics, EvolutionResult, EvolverConfig, ImprovementResult, PerformanceSnapshot,
    SelfEvolver,
};

pub use auto_evolve::AutoEvolve;
pub use auto_evolve::AutoEvolveConfig;
pub use auto_evolve::AutoEvolveResult;

pub use apex_core::{
    apex_fitness, apex_guidance, compute_apex, format_apex_state, ApexGuidance, ApexInput,
    ApexState,
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
