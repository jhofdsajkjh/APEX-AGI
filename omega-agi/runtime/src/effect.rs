//! # Runtime Effect System
//! Manages side effects in the actor system with lifecycle tracking.

use anyhow::Result;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

/// Unique effect identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EffectId(u64);

impl EffectId {
    pub fn new() -> Self {
        static COUNTER: AtomicU64 = AtomicU64::new(1);
        EffectId(COUNTER.fetch_add(1, Ordering::SeqCst))
    }
}

impl std::fmt::Display for EffectId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Effect#{}", self.0)
    }
}

/// Context for executing an effect.
#[derive(Debug, Clone)]
pub struct EffectContext {
    pub effect_id: EffectId,
    pub actor_id: Option<String>,
    pub correlation_id: Option<String>,
}

/// Result of an effect execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EffectResult {
    pub effect_id: EffectId,
    pub success: bool,
    pub output: Option<serde_json::Value>,
    pub error: Option<String>,
    pub duration_ms: u64,
}

/// A named effect with handler.
#[derive(Clone)]
pub struct Effect {
    pub name: String,
    pub handler: Arc<dyn Fn(serde_json::Value) -> Result<serde_json::Value> + Send + Sync>,
}

impl std::fmt::Debug for Effect {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Effect").field("name", &self.name).finish()
    }
}

/// The effect system registry and executor.
pub struct EffectSystem {
    effects: Arc<RwLock<HashMap<String, Effect>>>,
    history: Arc<RwLock<Vec<EffectResult>>>,
}

impl EffectSystem {
    pub fn new() -> Self {
        Self {
            effects: Arc::new(RwLock::new(HashMap::new())),
            history: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub fn register(&self, effect: Effect) {
        self.effects.write().insert(effect.name.clone(), effect);
    }

    pub fn execute(&self, name: &str, input: serde_json::Value) -> EffectResult {
        let effect_id = EffectId::new();
        let start = std::time::Instant::now();

        let result = match self.effects.read().get(name) {
            Some(effect) => match (effect.handler)(input) {
                Ok(output) => EffectResult {
                    effect_id,
                    success: true,
                    output: Some(output),
                    error: None,
                    duration_ms: start.elapsed().as_millis() as u64,
                },
                Err(e) => EffectResult {
                    effect_id,
                    success: false,
                    output: None,
                    error: Some(e.to_string()),
                    duration_ms: start.elapsed().as_millis() as u64,
                },
            },
            None => EffectResult {
                effect_id,
                success: false,
                output: None,
                error: Some(format!("Effect '{}' not registered", name)),
                duration_ms: 0,
            },
        };

        self.history.write().push(result.clone());
        result
    }

    pub fn list_effects(&self) -> Vec<String> {
        self.effects.read().keys().cloned().collect()
    }

    pub fn history(&self) -> Vec<EffectResult> {
        self.history.read().clone()
    }
}

impl Default for EffectSystem {
    fn default() -> Self {
        Self::new()
    }
}
