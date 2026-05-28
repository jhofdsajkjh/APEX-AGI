//! Auto-recovery — automatic system restoration
//!
//! Monitors system health and automatically triggers recovery
//! actions when failures are detected, with exponential backoff.

use std::sync::Arc;
use tokio::sync::RwLock;

/// Types of recovery actions
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum RecoveryAction {
    None,
    RestartSubsystem(String),
    ResetToLastGood,
    FullRestart,
    EmergencyShutdown,
}

impl RecoveryAction {
    pub fn description(&self) -> &str {
        match self {
            Self::None => "No recovery needed",
            Self::RestartSubsystem(s) => s,
            Self::ResetToLastGood => "Reset to last known good state",
            Self::FullRestart => "Full system restart initiated",
            Self::EmergencyShutdown => "EMERGENCY: System shutdown",
        }
    }
}

/// The RecoveryManager — automatic system recovery
pub struct RecoveryManager {
    auto_recovery: bool,
    last_action: Arc<RwLock<Option<RecoveryAction>>>,
    recovery_count: Arc<RwLock<u32>>,
    last_reason: Arc<RwLock<String>>,
}

impl RecoveryManager {
    pub fn new(auto_recovery: bool) -> Self {
        Self {
            auto_recovery,
            last_action: Arc::new(RwLock::new(None)),
            recovery_count: Arc::new(RwLock::new(0)),
            last_reason: Arc::new(RwLock::new(String::new())),
        }
    }

    /// Trigger a recovery action
    pub async fn trigger(&self, reason: &str) -> RecoveryAction {
        let count = {
            let mut c = self.recovery_count.write().await;
            *c += 1;
            *c
        };

        {
            let mut r = self.last_reason.write().await;
            *r = reason.to_string();
        }

        // Determine action based on severity and count
        let action = if !self.auto_recovery {
            RecoveryAction::None
        } else if count <= 2 {
            RecoveryAction::RestartSubsystem(format!("Subsystem restart (attempt {}/3): {}", count, reason))
        } else if count <= 5 {
            RecoveryAction::ResetToLastGood
        } else if count <= 8 {
            RecoveryAction::FullRestart
        } else {
            RecoveryAction::EmergencyShutdown
        };

        {
            let mut la = self.last_action.write().await;
            *la = Some(action.clone());
        }

        tracing::info!(?action, count, reason = %reason, "Recovery triggered");
        action
    }

    /// Get last recovery action
    pub async fn last_action(&self) -> Option<RecoveryAction> {
        self.last_action.read().await.clone()
    }

    /// Get recovery count
    pub async fn recovery_count(&self) -> u32 {
        *self.recovery_count.read().await
    }

    /// Reset recovery counter
    pub async fn reset(&self) {
        let mut c = self.recovery_count.write().await;
        *c = 0;
        let mut r = self.last_reason.write().await;
        r.clear();
    }
}
