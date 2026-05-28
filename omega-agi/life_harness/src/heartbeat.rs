//! Heartbeat monitoring — keeps the pulse of the system
//!
//! Continuously monitors system health through periodic pings,
//! tracks failures, and alerts when thresholds are exceeded.

use std::sync::Arc;
use tokio::sync::RwLock;

/// Status of the heartbeat monitor
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct HeartbeatStatus {
    pub last_ping: String,
    pub consecutive_successes: u32,
    pub failures: u32,
    pub total_pings: u32,
    pub healthy: bool,
    pub uptime_secs: u64,
}

/// The HeartbeatMonitor — keeps the system alive
pub struct HeartbeatMonitor {
    interval_secs: u64,
    max_failures: u32,
    last_ping: Arc<RwLock<String>>,
    successes: Arc<RwLock<u32>>,
    failures: Arc<RwLock<u32>>,
    total: Arc<RwLock<u32>>,
    start_time: std::time::Instant,
}

impl HeartbeatMonitor {
    pub fn new(interval_secs: u64, max_failures: u32) -> Self {
        Self {
            interval_secs,
            max_failures,
            last_ping: Arc::new(RwLock::new(String::new())),
            successes: Arc::new(RwLock::new(0)),
            failures: Arc::new(RwLock::new(0)),
            total: Arc::new(RwLock::new(0)),
            start_time: std::time::Instant::now(),
        }
    }

    /// Record a heartbeat ping
    pub async fn ping(&self) -> HeartbeatStatus {
        let now = chrono::Utc::now().to_rfc3339();
        {
            let mut last = self.last_ping.write().await;
            *last = now.clone();
        }
        {
            let mut s = self.successes.write().await;
            *s += 1;
        }
        {
            let mut t = self.total.write().await;
            *t += 1;
        }

        self.status().await
    }

    /// Record a heartbeat failure
    pub async fn fail(&self) -> HeartbeatStatus {
        {
            let mut f = self.failures.write().await;
            *f += 1;
        }
        self.status().await
    }

    /// Get current status
    pub async fn status(&self) -> HeartbeatStatus {
        let last = self.last_ping.read().await.clone();
        let successes = *self.successes.read().await;
        let failures = *self.failures.read().await;
        let total = *self.total.read().await;

        HeartbeatStatus {
            last_ping: if last.is_empty() { "never".into() } else { last },
            consecutive_successes: successes,
            failures,
            total_pings: total,
            healthy: failures < self.max_failures,
            uptime_secs: self.start_time.elapsed().as_secs(),
        }
    }

    pub fn interval(&self) -> u64 {
        self.interval_secs
    }
}
