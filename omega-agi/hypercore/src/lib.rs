//! # OMEGA HyperCore
//!
//! Zero-allocation async runtime with persistent memory and capability security.
//! Layer 0 of the OMEGA AGI system.

pub mod scheduler;
pub mod memory;
pub mod security;
pub mod session;
pub mod errors;
pub mod health;
pub mod logging;
pub mod pipeline;
pub mod diagnostics;
pub mod self_heal;

pub use scheduler::{TaskScheduler, TaskPriority, TaskId};
pub use memory::{MemoryPool, MemoryStats};
pub use security::{Capability, CapabilitySet, SecurityRing};
pub use session::{SessionConfig, SessionManager};
pub use errors::HyperCoreError;
pub use health::{HealthMonitor, HealthSnapshot};
pub use pipeline::{PipelineOrchestrator, PipelineResult, HealthCheck};
pub use diagnostics::{DiagnosticEngine, SystemHealthReport, SubsystemHealth};
pub use self_heal::{SelfHealingController, HealingAction, HealingResult, HealingEvent, Healer};

/// HyperCore version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Top-level HyperCore wrapper providing integrated access to all subsystems.
pub struct HyperCore {
    pub scheduler: TaskScheduler,
    pub memory: MemoryPool,
    pub security: SecurityRing,
    pub session: SessionManager,
    pub health: HealthMonitor,
    pub diagnostics: DiagnosticEngine,
    pub pipeline: PipelineOrchestrator,
    pub healing: SelfHealingController,
    pub logger: logging::Logger,
}

impl HyperCore {
    pub fn new() -> anyhow::Result<Self> {
        Ok(Self {
            scheduler: TaskScheduler::new(),
            memory: MemoryPool::new(1024 * 1024 * 10)?,
            security: SecurityRing::default(),
            session: SessionManager::new(SessionConfig::default()),
            health: HealthMonitor::new(),
            diagnostics: DiagnosticEngine::new(),
            pipeline: PipelineOrchestrator::new("omega"),
            healing: SelfHealingController::new(),
            logger: logging::Logger::default(),
        })
    }
}
