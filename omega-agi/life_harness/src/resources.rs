//! Resource monitoring — system resource tracking
//!
//! Monitors CPU, memory, disk, and network resources.
//! Uses the sysinfo crate for real system metrics.

use std::sync::{Arc, Mutex};
use sysinfo::{Disks, Networks, System, MINIMUM_CPU_UPDATE_INTERVAL};
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
    system: Arc<Mutex<System>>,
    initialized: Arc<Mutex<bool>>,
}

impl ResourceMonitor {
    pub fn new(interval: u64) -> Self {
        Self {
            interval,
            history: Arc::new(RwLock::new(Vec::new())),
            current: Arc::new(RwLock::new(None)),
            system: Arc::new(Mutex::new(System::new_all())),
            initialized: Arc::new(Mutex::new(false)),
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
            if sample.network_active {
                "active"
            } else {
                "idle"
            },
        )
    }

    async fn collect(&self) -> ResourceSample {
        let mut system = self.system.lock().unwrap();
        let mut initialized = self.initialized.lock().unwrap();

        // CPU needs two samples for delta calculation
        if !*initialized {
            // First call: just seed the data
            system.refresh_cpu_usage();
            std::thread::sleep(MINIMUM_CPU_UPDATE_INTERVAL);
            system.refresh_cpu_usage();
            *initialized = true;
        } else {
            system.refresh_cpu_usage();
        }

        let cpu_percent = system.global_cpu_usage() as f64;

        // Memory (bytes → MB)
        system.refresh_memory();
        let used_bytes = system.total_memory() - system.available_memory();
        let memory_mb = used_bytes as f64 / 1024.0 / 1024.0;

        // Disk — use Disks API directly
        let disks = Disks::new_with_refreshed_list();
        let disk_gb: f64 = disks
            .list()
            .iter()
            .map(|d| d.total_space() as f64)
            .sum::<f64>()
            / 1_000_000_000.0;

        // Network — active if any interface exists (basic check)
        let networks = Networks::new_with_refreshed_list();
        let network_active = networks.list().iter().count() > 0;

        ResourceSample {
            timestamp: chrono::Utc::now().to_rfc3339(),
            cpu_percent,
            memory_mb,
            disk_gb,
            network_active,
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
