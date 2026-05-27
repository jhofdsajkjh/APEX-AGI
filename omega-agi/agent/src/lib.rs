//! # OMEGA AGI Agent — Layer 5
//!
//! The LLM-powered autonomous agent that sits on top of the five-layer OMEGA AGI
//! system. It uses ReAct (Reasoning + Acting) to understand user tasks, call tools
//! that bridge into every layer, and produce results.
//!
//! ## Architecture
//!
//! ```text
//! User Task
//!     │
//!     ▼
//! ┌─────────────────────┐
//! │   Agent (this crate)│  ←  ReAct loop (Thought → Action → Observation)
//! │   ┌───────────────┐ │
//! │   │   LLM Client  │ │  ←  OpenAI-compatible HTTP client
//! │   └──────┬────────┘ │
//! │   ┌──────▼────────┐ │
//! │   │  Tool Context │ │  ←  Trait implemented by host (main.rs)
//! │   └──────┬────────┘ │
//! └──────────┼──────────┘
//!            ▼
//!     OMEGA AGI Layers (HyperCore, Runtime, Engineering, Evolution, Adapters)
//! ```

pub mod llm;
pub mod react;
pub mod tools;

use llm::LLMClient;
use react::ReActEngine;
use tools::ToolContext;
use std::sync::Arc;

/// The top-level Agent — combines an LLM client with a ReAct engine and
/// a tool context that bridges into the five OMEGA AGI layers.
pub struct Agent {
    llm: LLMClient,
    react: ReActEngine,
    context: Arc<dyn ToolContext>,
}

impl Agent {
    /// Create a new Agent with the given LLM client, context, and default settings.
    pub fn new(llm: LLMClient, context: Arc<dyn ToolContext>) -> Self {
        Self { llm, react: ReActEngine::default(), context }
    }

    /// Create an Agent from environment variables (`.env` file).
    /// Calls `LLMClient::from_env()`.
    pub fn from_env(context: Arc<dyn ToolContext>) -> Self {
        Self::new(LLMClient::from_env(), context)
    }

    /// Run a task through the ReAct loop.
    ///
    /// The agent will reason, call tools, and produce a final answer.
    pub async fn run(&self, task: &str) -> anyhow::Result<String> {
        tracing::info!(task = %task, "Agent starting task");
        self.react.run(&self.llm, task, self.context.as_ref()).await
    }

    /// Quick shorthand: run a task and print the result via `tracing::info`.
    pub async fn run_and_log(&self, task: &str) -> anyhow::Result<String> {
        let result = self.run(task).await?;
        tracing::info!(task = %task, result = %result, "Agent completed task");
        Ok(result)
    }
}
