//! # Omega Evolution Library
//! Layer 3 - Self-evolution engine for continuous performance improvement.

pub mod auto_evolve;
pub mod self_evolve;

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

/// Evolution engine version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

use std::sync::Mutex;

/// Top-level evolution module wrapper.
pub struct Evolution {
    pub evolver: Mutex<SelfEvolver>,
    pub config: EvolverConfig,
}

impl Evolution {
    pub fn new() -> Self {
        let config = EvolverConfig::default();
        Self {
            evolver: Mutex::new(SelfEvolver::new(config.clone())),
            config,
        }
    }

    pub fn with_config(config: EvolverConfig) -> Self {
        Self {
            evolver: Mutex::new(SelfEvolver::new(config.clone())),
            config,
        }
    }

    /// Run one evolution cycle (thread-safe, locks the evolver internally).
    pub fn evolve(&self) -> EvolutionResult {
        self.evolver.lock().unwrap().evolve()
    }

    /// Access the locked evolver for custom operations.
    pub fn lock_evolver(&self) -> std::sync::MutexGuard<'_, SelfEvolver> {
        self.evolver.lock().unwrap()
    }
}

impl Default for Evolution {
    fn default() -> Self {
        Self::new()
    }
}
