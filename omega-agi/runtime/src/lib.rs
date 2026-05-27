//! # OMEGA Runtime
//!
//! Layer 1 execution engine for the OMEGA AGI system.
//! Provides actor system, effect system, WASM sandbox, ML inference, and graph execution.
//!
//! Built on top of `omega-hypercore` for scheduling, memory, security, and session management.

pub mod actor;
pub mod effect;
pub mod wasm_sandbox;
pub mod ml_inference;
pub mod graph_executor;
pub mod swarm;

pub use actor::{Actor, ActorId, ActorRef, ActorSystem, Message};
pub use effect::{Effect, EffectContext, EffectId, EffectResult, EffectSystem};
pub use wasm_sandbox::{WasmError, WasmModule, WasmSandbox, WasmSandboxConfig};
pub use ml_inference::{InferenceConfig, InferenceEngine, InferenceResult, ModelHandle};
pub use graph_executor::{GraphExecutor, GraphExecutorError, NodeId, NodeResult, TaskGraph};

/// Runtime version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Top-level Runtime wrapper providing integrated access to all subsystems.
pub struct Runtime {
    pub actor_system: ActorSystem,
    pub wasm_sandbox: WasmSandbox,
    pub effects: EffectSystem,
    pub inference: InferenceEngine,
    pub graph_executor: GraphExecutor,
}

impl Runtime {
    pub fn new() -> anyhow::Result<Self> {
        Ok(Self {
            actor_system: ActorSystem::new(),
            wasm_sandbox: WasmSandbox::with_defaults(),
            effects: EffectSystem::new(),
            inference: InferenceEngine::with_defaults(),
            graph_executor: GraphExecutor::new(),
        })
    }
}


