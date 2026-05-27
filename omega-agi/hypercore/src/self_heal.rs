//! # HyperCore Self-Healing
//! Automatic healing controller for detecting and recovering from failures.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A healing action that can be applied.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HealingAction {
    Restart { target: String },
    Recover { target: String, strategy: String },
    Scale { target: String, replicas: u32 },
    Clear { target: String },
    Rollback { target: String, version: String },
}

/// Result of a healing operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealingResult {
    pub action: String,
    pub success: bool,
    pub message: String,
    pub duration_ms: u64,
}

/// A healing event recorded in the event log.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealingEvent {
    pub id: u64,
    pub timestamp: DateTime<Utc>,
    pub target: String,
    pub action: String,
    pub success: bool,
    pub details: String,
}

/// Trait for healers that can handle specific failure modes.
pub trait Healer: Send + Sync {
    fn name(&self) -> &str;
    fn can_handle(&self, issue: &str) -> bool;
    fn heal(&self, target: &str, issue: &str) -> HealingResult;
}

/// Generic healer for simple service recovery.
pub struct GenericHealer {
    name: String,
    handled_issues: Vec<String>,
}

impl GenericHealer {
    pub fn new(name: &str, issues: &[&str]) -> Self {
        Self {
            name: name.to_string(),
            handled_issues: issues.iter().map(|s| s.to_string()).collect(),
        }
    }
}

impl Healer for GenericHealer {
    fn name(&self) -> &str {
        &self.name
    }

    fn can_handle(&self, issue: &str) -> bool {
        self.handled_issues.iter().any(|i| issue.contains(i))
    }

    fn heal(&self, target: &str, _issue: &str) -> HealingResult {
        tracing::info!(healer = %self.name, target = %target, "Performing generic heal");
        HealingResult {
            action: format!("generic_recover({})", target),
            success: true,
            message: format!("Recovery action applied to {}", target),
            duration_ms: 0,
        }
    }
}

/// Self-healing controller that manages healers and healing events.
pub struct SelfHealingController {
    healers: Vec<Box<dyn Healer>>,
    events: Vec<HealingEvent>,
    event_counter: AtomicU64,
}

impl SelfHealingController {
    pub fn new() -> Self {
        Self {
            healers: Vec::new(),
            events: Vec::new(),
            event_counter: AtomicU64::new(1),
        }
    }

    pub fn register_healer(&mut self, healer: Box<dyn Healer>) {
        tracing::info!(healer = %healer.name(), "Self-healer registered");
        self.healers.push(healer);
    }

    pub fn diagnose_and_heal(&mut self, target: &str, issue: &str) -> HealingResult {
        for healer in &self.healers {
            if healer.can_handle(issue) {
                let result = healer.heal(target, issue);
                self.events.push(HealingEvent {
                    id: self.event_counter.fetch_add(1, Ordering::SeqCst),
                    timestamp: Utc::now(),
                    target: target.to_string(),
                    action: result.action.clone(),
                    success: result.success,
                    details: result.message.clone(),
                });
                return result;
            }
        }

        HealingResult {
            action: "noop".to_string(),
            success: false,
            message: format!("No healer found for issue: {} on target: {}", issue, target),
            duration_ms: 0,
        }
    }

    pub fn get_events(&self) -> &[HealingEvent] {
        &self.events
    }

    pub fn healer_count(&self) -> usize {
        self.healers.len()
    }
}

impl Default for SelfHealingController {
    fn default() -> Self {
        Self::new()
    }
}
