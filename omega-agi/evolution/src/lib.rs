//! # Omega Evolution Library
//! Layer 3 - Self-evolution engine for continuous performance improvement.

pub mod self_evolve;

pub use self_evolve::{
    SelfEvolver,
    EvolverConfig,
    EvolutionResult,
    EvolutionMetrics,
    ImprovementResult,
    PerformanceSnapshot,
};

/// Evolution engine version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Top-level evolution module wrapper.
pub struct Evolution {
    pub evolver: SelfEvolver,
    pub config: EvolverConfig,
}

impl Evolution {
    pub fn new() -> Self {
        let config = EvolverConfig::default();
        Self {
            evolver: SelfEvolver::new(config.clone()),
            config,
        }
    }

    pub fn with_config(config: EvolverConfig) -> Self {
        Self {
            evolver: SelfEvolver::new(config.clone()),
            config,
        }
    }
}

impl Default for Evolution {
    fn default() -> Self {
        Self::new()
    }
}
