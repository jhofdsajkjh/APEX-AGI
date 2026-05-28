//! Performance boost modes — amplify system capabilities
//!
//! Multiple performance modes from Eco to Overdrive that tune
//! system behavior for different operational priorities.

use std::sync::Arc;
use tokio::sync::RwLock;

/// Performance boost modes
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum BoostMode {
    Eco,
    Normal,
    Turbo,
    Overdrive,
}

impl BoostMode {
    pub fn name(&self) -> &str {
        match self {
            Self::Eco => "🌱 Eco",
            Self::Normal => "⚡ Normal",
            Self::Turbo => "🔥 Turbo",
            Self::Overdrive => "🚀 Overdrive",
        }
    }

    pub fn multiplier(&self) -> f64 {
        match self {
            Self::Eco => 0.5,
            Self::Normal => 1.0,
            Self::Turbo => 1.8,
            Self::Overdrive => 3.0,
        }
    }

    pub fn description(&self) -> &str {
        match self {
            Self::Eco => "Power saving mode, reduced resource usage",
            Self::Normal => "Balanced performance and efficiency",
            Self::Turbo => "High performance, increased resource usage",
            Self::Overdrive => "Maximum performance, all resources allocated",
        }
    }
}

/// The BoostManager — controls performance modes
pub struct BoostManager {
    current: Arc<RwLock<BoostMode>>,
    history: Arc<RwLock<Vec<(String, BoostMode)>>>,
}

impl BoostManager {
    pub fn new(default: BoostMode) -> Self {
        Self {
            current: Arc::new(RwLock::new(default)),
            history: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Activate a boost mode
    pub fn activate(&self, mode: BoostMode) -> String {
        let old_mode = {
            let mut current = self.current.blocking_write();
            let old = *current;
            *current = mode;
            old
        };

        {
            let mut history = self.history.blocking_write();
            history.push((chrono::Utc::now().to_rfc3339(), mode));
            if history.len() > 100 {
                history.remove(0);
            }
        }

        format!(
            "Boost mode changed: {} → {} ({}x)",
            old_mode.name(),
            mode.name(),
            mode.multiplier(),
        )
    }

    /// Get current boost mode
    pub fn current(&self) -> BoostMode {
        *self.current.blocking_read()
    }

    /// Cycle to next boost mode
    pub fn cycle(&self) -> String {
        let next = match self.current() {
            BoostMode::Eco => BoostMode::Normal,
            BoostMode::Normal => BoostMode::Turbo,
            BoostMode::Turbo => BoostMode::Overdrive,
            BoostMode::Overdrive => BoostMode::Eco,
        };
        self.activate(next)
    }
}
