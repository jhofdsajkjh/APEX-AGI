//! Resource monitoring — system resource tracking
//!
//! Monitors CPU, memory, disk, and network resources.
//! Uses simulated metrics with a real-time sampling approach.

use std::sync::Arc;
use tokio::sync::RwLock;

/// A resource sample at a point in time
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ResourceSample {
    pub timestamp: String,
    pub cpu_percent: f64,
    pub memory_mb: f64,
    pub disk_gb: f64,
    pub network_active: bool,
}

/// The ResourceMonitor — tracks system resources
pub struct ResourceMonitor {
    interval: u64,
    history: Arc<RwLock<Vec<ResourceSample>>>,
    current: Arc<RwLock<Option<ResourceSample>>>,
}

impl ResourceMonitor {
    pub fn new(interval: u64) -> Self {
        Self {
            interval,
            history: Arc::new(RwLock::new(Vec::new())),
            current: Arc::new(RwLock::new(None)),
        }
    }

    /// Take a resource snapshot
    pub async fn snapshot(&self) -> String {
        let sample = self.collect().await;
        let mut history = self.history.write().await;
        history.push(sample.clone());
        if history.len() > 100 {
            history.remove(0);
        }

        format!(
            "CPU: {:.1}% | Memory: {:.0} MB | Disk: {:.1} GB | Network: {}",
            sample.cpu_percent,
            sample.memory_mb,
            sample.disk_gb,
            if sample.network_active { "active" } else { "idle" },
        )
    }

    async fn collect(&self) -> ResourceSample {
        // Simulated resource metrics (in production, use sysinfo crate)
        use std::time::{SystemTime, UNIX_EPOCH};

        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default();
        let seed = now.as_nanos();

        // Deterministic pseudo-random variation based on time
        let variation = |base: f64, amp: f64| -> f64 {
            let phase = (seed % 1000) as f64 / 1000.0;
            base + (phase * 2.0 - 1.0) * amp
        };

        ResourceSample {
            timestamp: chrono::Utc::now().to_rfc3339(),
            cpu_percent: variation(45.0, 30.0).max(0.0).min(100.0),
            memory_mb: variation(512.0, 256.0).max(64.0),
            disk_gb: variation(50.0, 10.0).max(1.0),
            network_active: seed % 3 != 0,
        }
    }

    /// Get sensor interval
    pub fn interval(&self) -> u64 {
        self.interval
    }

    /// Get average CPU over history
    pub async fn avg_cpu(&self) -> f64 {
        let history = self.history.read().await;
        if history.is_empty() {
            return 0.0;
        }
        history.iter().map(|s| s.cpu_percent).sum::<f64>() / history.len() as f64
    }
}
