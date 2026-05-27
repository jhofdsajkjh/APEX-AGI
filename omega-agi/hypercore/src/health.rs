//! # HyperCore Health Monitor
//! Tracks subsystem health and provides health snapshots.

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// Health status of a single subsystem.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubsystemHealth {
    pub name: String,
    pub healthy: bool,
    pub last_heartbeat: DateTime<Utc>,
    pub error_count: u64,
    pub message: String,
}

impl SubsystemHealth {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            healthy: true,
            last_heartbeat: Utc::now(),
            error_count: 0,
            message: "OK".to_string(),
        }
    }
}

/// A point-in-time snapshot of overall system health.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthSnapshot {
    pub timestamp: DateTime<Utc>,
    pub overall_healthy: bool,
    pub subsystems: Vec<SubsystemHealth>,
    pub uptime_seconds: u64,
}

/// Monitors and reports on system health.
pub struct HealthMonitor {
    healthy: AtomicBool,
    subsystems: Arc<RwLock<HashMap<String, SubsystemHealth>>>,
    start_time: DateTime<Utc>,
}

impl HealthMonitor {
    pub fn new() -> Self {
        Self {
            healthy: AtomicBool::new(true),
            subsystems: Arc::new(RwLock::new(HashMap::new())),
            start_time: Utc::now(),
        }
    }

    pub fn register_subsystem(&self, name: &str) {
        let mut subs = self.subsystems.write();
        subs.insert(name.to_string(), SubsystemHealth::new(name));
    }

    pub fn report_healthy(&self, name: &str) {
        let mut subs = self.subsystems.write();
        if let Some(h) = subs.get_mut(name) {
            h.healthy = true;
            h.last_heartbeat = Utc::now();
            h.message = "OK".to_string();
        }
    }

    pub fn report_unhealthy(&self, name: &str, message: &str) {
        let mut subs = self.subsystems.write();
        if let Some(h) = subs.get_mut(name) {
            h.healthy = false;
            h.last_heartbeat = Utc::now();
            h.error_count += 1;
            h.message = message.to_string();
        }
        self.healthy.store(false, Ordering::SeqCst);
    }

    pub fn snapshot(&self) -> HealthSnapshot {
        let subs = self.subsystems.read();
        let all_healthy = subs.values().all(|h| h.healthy);
        HealthSnapshot {
            timestamp: Utc::now(),
            overall_healthy: all_healthy,
            subsystems: subs.values().cloned().collect(),
            uptime_seconds: (Utc::now() - self.start_time).num_seconds().max(0) as u64,
        }
    }

    pub fn is_healthy(&self) -> bool {
        self.healthy.load(Ordering::SeqCst)
    }
}

impl Default for HealthMonitor {
    fn default() -> Self {
        Self::new()
    }
}
