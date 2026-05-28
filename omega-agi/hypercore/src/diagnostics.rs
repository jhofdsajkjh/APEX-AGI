//! # HyperCore Diagnostics
//! System diagnostic engine for health analysis and reporting.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Health status of a subsystem within a diagnostic report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubsystemHealth {
    pub name: String,
    pub status: String,
    pub details: String,
}

/// Full system health report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemHealthReport {
    pub timestamp: DateTime<Utc>,
    pub overall_status: String,
    pub subsystems: Vec<SubsystemHealth>,
    pub recommendation: String,
}

/// Diagnostic engine for analyzing system health.
pub struct DiagnosticEngine {
    subsystems: HashMap<String, String>,
}

impl DiagnosticEngine {
    pub fn new() -> Self {
        Self {
            subsystems: HashMap::new(),
        }
    }

    pub fn register_subsystem(&mut self, name: &str) {
        self.subsystems
            .insert(name.to_string(), "healthy".to_string());
    }

    pub fn run_diagnostics(&self) -> SystemHealthReport {
        let subsystems: Vec<SubsystemHealth> = self
            .subsystems
            .iter()
            .map(|(name, status)| SubsystemHealth {
                name: name.clone(),
                status: status.clone(),
                details: format!("Subsystem '{}' is {}", name, status),
            })
            .collect();

        let all_healthy = subsystems.iter().all(|s| s.status == "healthy");
        let overall = if all_healthy {
            "healthy".to_string()
        } else {
            "degraded".to_string()
        };

        SystemHealthReport {
            timestamp: Utc::now(),
            overall_status: overall,
            recommendation: if all_healthy {
                "All systems operational".to_string()
            } else {
                "Review degraded subsystems".to_string()
            },
            subsystems,
        }
    }

    pub fn get_subsystem_count(&self) -> usize {
        self.subsystems.len()
    }
}

impl Default for DiagnosticEngine {
    fn default() -> Self {
        Self::new()
    }
}
