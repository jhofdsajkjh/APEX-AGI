//! # OMEGA AGI - Life-Harness Engine (Layer 7)
//!
//! System self-maintenance layer ensuring 24/7 autonomous operation:
//! - **Heartbeat**: periodic system health monitoring
//! - **Recovery**: automatic restart & backoff strategies
//! - **Resources**: CPU, memory, disk monitoring
//! - **Persistence**: session state save/restore
//!
//! ## Architecture
//!
//! ```text
//! LifeHarness
//! ├── Heartbeat   (health monitor + alert)
//! ├── Recovery    (auto-restart + backoff)
//! ├── Resources   (CPU/mem/disk tracking)
//! └── Persistence (state save/restore)
//! ```

pub mod heartbeat;
pub mod persistence;
pub mod recovery;
pub mod resources;

use heartbeat::{HeartbeatMonitor, HeartbeatStatus};
use persistence::SessionStore;
use recovery::{RecoveryAction, RecoveryManager};
use resources::ResourceMonitor;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Life-Harness configuration
#[derive(Debug, Clone)]
pub struct LifeConfig {
    /// Heartbeat interval in seconds
    pub heartbeat_interval: u64,
    /// Max consecutive failures before recovery
    pub max_failures: u32,
    /// Auto-recovery enabled
    pub auto_recovery: bool,
    /// Resource monitoring interval in seconds
    pub resource_interval: u64,
    /// Persistence file path
    pub persistence_path: String,
    /// Enable session persistence
    pub enable_persistence: bool,
    /// Alert webhook URL (optional)
    pub alert_webhook: Option<String>,
}

impl Default for LifeConfig {
    fn default() -> Self {
        Self {
            heartbeat_interval: 10,
            max_failures: 3,
            auto_recovery: true,
            resource_interval: 30,
            persistence_path: "./data/life_harness_state.json".to_string(),
            enable_persistence: true,
            alert_webhook: None,
        }
    }
}

/// Overall system health summary
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct HealthSummary {
    pub status: SystemStatus,
    pub uptime_secs: u64,
    pub heartbeat: HeartbeatStatus,
    pub resources: String,
    pub last_recovery: Option<RecoveryAction>,
    pub sessions_active: usize,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub enum SystemStatus {
    Healthy,
    Degraded,
    Critical,
    Recovering,
}

/// The main Life-Harness — keeps the system alive and self-sustaining
pub struct LifeHarness {
    config: LifeConfig,
    heartbeat: HeartbeatMonitor,
    recovery: RecoveryManager,
    resources: ResourceMonitor,
    persistence: SessionStore,
    start_time: std::time::Instant,
    warnings: Arc<RwLock<Vec<String>>>,
    errors: Arc<RwLock<Vec<String>>>,
}

impl LifeHarness {
    pub fn new() -> Self {
        Self::with_config(LifeConfig::default())
    }

    pub fn with_config(config: LifeConfig) -> Self {
        let persistence = SessionStore::new(&config.persistence_path);
        Self {
            config: config.clone(),
            heartbeat: HeartbeatMonitor::new(config.heartbeat_interval, config.max_failures),
            recovery: RecoveryManager::new(config.auto_recovery),
            resources: ResourceMonitor::new(config.resource_interval),
            persistence,
            start_time: std::time::Instant::now(),
            warnings: Arc::new(RwLock::new(Vec::new())),
            errors: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Start the life-harness monitoring loop
    pub async fn start(&self) -> anyhow::Result<()> {
        tracing::info!("Life-Harness starting...");

        // Restore previous session state if available
        if self.config.enable_persistence {
            if let Ok(state) = self.persistence.restore().await {
                tracing::info!("Restored {} sessions from persistence", state.len());
            }
        }

        // Initial heartbeat
        self.heartbeat.ping().await;
        tracing::info!("Life-Harness operational");
        Ok(())
    }

    /// Record a successful heartbeat
    pub async fn heartbeat(&self) -> HeartbeatStatus {
        let status = self.heartbeat.ping().await;
        if status.failures >= self.config.max_failures && self.config.auto_recovery {
            let action = self
                .recovery
                .trigger("Heartbeat failure threshold reached")
                .await;
            tracing::warn!("Recovery triggered: {:?}", action);
        }
        status
    }

    /// Get current resource snapshot
    pub async fn resource_snapshot(&self) -> String {
        self.resources.snapshot().await
    }

    /// Register a warning
    pub async fn warn(&self, msg: &str) {
        let mut w = self.warnings.write().await;
        w.push(format!("[{}] {}", chrono::Utc::now().to_rfc3339(), msg));
        if w.len() > 100 {
            w.remove(0);
        }
        tracing::warn!("Life-Harness warning: {}", msg);
    }

    /// Register an error
    pub async fn error(&self, msg: &str) {
        let mut e = self.errors.write().await;
        e.push(format!("[{}] {}", chrono::Utc::now().to_rfc3339(), msg));
        if e.len() > 50 {
            e.remove(0);
        }
        tracing::error!("Life-Harness error: {}", msg);

        if self.config.auto_recovery {
            let action = self.recovery.trigger(msg).await;
            tracing::info!("Auto-recovery triggered: {:?}", action);
        }
    }

    /// Trigger manual recovery
    pub async fn recover(&self, reason: &str) -> RecoveryAction {
        self.recovery.trigger(reason).await
    }

    /// Get comprehensive health summary
    pub async fn health(&self) -> HealthSummary {
        let uptime = self.start_time.elapsed().as_secs();
        let hb = self.heartbeat.status().await;
        let w = self.warnings.read().await.clone();
        let e = self.errors.read().await.clone();
        let resources = self.resources.snapshot().await;
        let sessions = self.persistence.session_count().await;

        let status = if !e.is_empty() {
            SystemStatus::Critical
        } else if hb.failures > 0 || !w.is_empty() {
            SystemStatus::Degraded
        } else {
            SystemStatus::Healthy
        };

        HealthSummary {
            status,
            uptime_secs: uptime,
            heartbeat: hb,
            resources,
            last_recovery: self.recovery.last_action().await,
            sessions_active: sessions,
            warnings: w,
            errors: e,
        }
    }

    /// Persist current session state
    pub async fn persist(&self) -> anyhow::Result<()> {
        if self.config.enable_persistence {
            self.persistence.save().await?;
        }
        Ok(())
    }

    /// Version string
    pub fn version() -> &'static str {
        env!("CARGO_PKG_VERSION")
    }
}

impl Default for LifeHarness {
    fn default() -> Self {
        Self::new()
    }
}
