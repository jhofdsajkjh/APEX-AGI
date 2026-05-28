//! # OMEGA AGI Agent — Layer 5
//!
//! LLM-powered autonomous agent with:
//! - **Multi-backend inference** (OpenAI, Mock, Candle, Router)
//! - **ReAct reasoning** (Thought → Action → Observation → Answer)
//! - **Tool system** (trait-based, extensible)
//! - **Long-term memory** (SQLite + vector search)
//! - **Knowledge base** (chunk + embed + recall)
//! - **Multi-agent orchestration** (coordinated specialists)
//! - **Feedback loops** (self-improvement via evolution)
//!
//! ## Architecture
//!
//! ```text
//! Agent
//! ├── InferenceEngine (trait: OpenAI / Mock / Candle / Router)
//! ├── ToolRegistry    (trait: Think / Read / Write / Bash / …)
//! ├── LongTermMemory  (SQLite + vector embedding)
//! ├── KnowledgeBase   (chunk + embed + graph)
//! ├── FeedbackCollector → SelfEvolver
//! └── Orchestrator    (multi-agent coordination)
//! ```

pub mod inference;
pub mod engines;
pub mod memory;
pub mod knowledge;
pub mod tool;
pub mod feedback;
pub mod orchestrator;

// Re-export old modules for backward compatibility
pub mod llm;
pub mod react;
pub mod tools;

use inference::InferenceEngine;
use tool::ToolRegistry;
use memory::{LongTermMemory, MemoryConfig, MemoryType};
use feedback::{FeedbackCollector, FeedbackIntegrator};
use std::sync::Arc;

/// The top-level Agent — combines an LLM engine with ReAct reasoning,
/// tools, memory, knowledge, and feedback.
pub struct Agent {
    /// Inference engine (OpenAI / Mock / Router / etc.)
    pub engine: Arc<dyn InferenceEngine>,
    /// Tool registry for discoverable tool execution
    pub tools: ToolRegistry,
    /// Long-term memory with vector search
    pub memory: LongTermMemory,
    /// Knowledge base for structured recall
    pub feedback_collector: Option<Arc<FeedbackCollector>>,
    /// Feedback integrator for evolution loop
    pub feedback_integrator: Option<FeedbackIntegrator>,
    /// Agent identity
    pub name: String,
    /// Max ReAct steps
    max_steps: usize,
}

impl Agent {
    /// Create a new Agent with the given engine and tools.
    pub fn new(
        name: impl Into<String>,
        engine: Arc<dyn InferenceEngine>,
        tools: ToolRegistry,
        memory_config: MemoryConfig,
    ) -> Self {
        let memory = LongTermMemory::new(memory_config, engine.clone(), &name);
        Self {
            engine,
            tools,
            memory,
            feedback_collector: None,
            feedback_integrator: None,
            name: name.into(),
            max_steps: 15,
        }
    }

    /// Create from environment (backward-compatible).
    pub fn from_env(tools: ToolRegistry) -> Self {
        let engine: Arc<dyn InferenceEngine> = if std::env::var("OMEGA_API_KEY").ok()
            .and_then(|k| if k.is_empty() { None } else { Some(k) })
            .is_some()
        {
            Arc::new(engines::OpenAIEngine::from_env())
        } else {
            tracing::warn!("OMEGA_API_KEY not set — using MockEngine. Set in .env for real LLM access.");
            Arc::new(engines::MockEngine::new())
        };

        let memory_config = MemoryConfig {
            db_path: std::env::var("OMEGA_MEMORY_PATH")
                .unwrap_or_else(|_| "./data/memory.db".into()),
            ..Default::default()
        };

        Self::new("omega-agent", engine, tools, memory_config)
    }

    /// Attach a feedback collector (enables self-improvement loop).
    pub fn with_feedback(mut self, collector: Arc<FeedbackCollector>) -> Self {
        self.feedback_collector = Some(collector.clone());
        self.feedback_integrator = Some(FeedbackIntegrator::new(collector));
        self
    }

    /// Set max ReAct steps.
    pub fn with_max_steps(mut self, steps: usize) -> Self {
        self.max_steps = steps;
        self
    }

    /// Run the Agent on a task using the ReAct loop.
    pub async fn run(&self, task: &str) -> anyhow::Result<String> {
        tracing::info!(agent = %self.name, task = %task, "Starting task");

        let system_prompt = self.build_system_prompt();
        let mut messages = vec![
            inference::Message::system(&system_prompt),
            inference::Message::user(task),
        ];

        for step in 0..self.max_steps {
            let resp = self.engine.chat(&messages).await?;
            let content = resp.content.clone();

            tracing::debug!(step, response_len = content.len(), "LLM response");
            messages.push(inference::Message::assistant(&content));

            // Try Answer first
            if let Some(answer) = self.parse_answer(&content) {
                tracing::info!(agent = %self.name, step, "Task completed");
                // Store successful procedure in memory
                self.memory.store_tagged(
                    format!("Task completed: {}\nResult: {}", task, answer),
                    MemoryType::Procedural,
                    0.8,
                    vec!["success".into(), self.name.clone()],
                ).await;
                return Ok(answer);
            }

            // Try Action
            if let Some((action, args_json)) = self.parse_action(&content) {
                let args: serde_json::Value =
                    serde_json::from_str(&args_json).unwrap_or(serde_json::Value::Object(Default::default()));

                tracing::debug!(step, action = %action, args = %args_json, "Executing tool");

                let result = self.tools.execute(&action, &args).await;
                let obs = if result.success {
                    format!("Observation:\n{}", self.truncate(&result.output, 4000))
                } else {
                    format!("Observation (error):\n{}", self.truncate(&result.output, 2000))
                };

                tracing::debug!(step, tool_success = result.success, "Tool result");
                messages.push(inference::Message::user(&obs));

                // Store in memory
                self.memory.store_tagged(
                    format!("Step {}: Action {} → {}", step, action, if result.success { "success" } else { "failed" }),
                    MemoryType::Episodic,
                    0.4,
                    vec![action, self.name.clone()],
                ).await;
            } else {
                // Parse failure — help the LLM reformat
                messages.push(inference::Message::user(
                    "Format error: Your response must contain either:\n\
                     1. Thought: <reasoning>\n   Action: <tool_name>\n   Action Input: <JSON args>\n\
                     OR\n\
                     2. Thought: <reasoning>\n   Answer: <final answer>\n\n\
                     Please try again with the correct format."
                ));
            }
        }

        // Max steps reached — force a final answer
        messages.push(inference::Message::user(
            "You have reached the maximum number of reasoning steps. \
             Please provide your final answer now based on what you've learned so far."
        ));
        let final_resp = self.engine.chat(&messages).await?;
        Ok(self.parse_answer(&final_resp.content).unwrap_or_else(|| {
            format!("Maximum steps reached ({}). Last response:\n{}", self.max_steps, final_resp.content)
        }))
    }

    fn build_system_prompt(&self) -> String {
        format!(
            r#"You are **{}** — an autonomous AI agent in the OMEGA AGI system.

Your role: understand the user's task, reason step by step, and use the available
tools to accomplish it. You can also use `think` as a tool for internal reasoning.

{}
Important rules:
1. Always start with `Thought:` explaining your reasoning.
2. Then use exactly one `Action:` + `Action Input:` per turn.
3. Wait for the `Observation:` result before thinking again.
4. When you have enough information, produce `Answer:` with the final response.
5. If a tool fails, try an alternative approach or explain the error.
6. Be thorough and precise.
"#,
            self.name,
            self.tools.system_prompt()
        )
    }

    fn parse_answer(&self, text: &str) -> Option<String> {
        let idx = text.find("Answer:")?;
        let after = &text[idx + "Answer:".len()..];
        // If there's a "Action:" after Answer:, truncate
        let end = after.find("\nAction:").unwrap_or(after.len());
        let end = after[..end].find("\n\n").unwrap_or(end);
        Some(after[..end].trim().to_string())
    }

    fn parse_action(&self, text: &str) -> Option<(String, String)> {
        let action_idx = text.find("Action:")?;
        let after_action = &text[action_idx + "Action:".len()..];
        let action_line = after_action.lines().next()?.trim();
        // Split on possible whitespace
        let action = action_line.split_whitespace().next()?.to_lowercase();

        // Find Action Input
        let input_idx = text.find("Action Input:")?;
        let after_input = &text[input_idx + "Action Input:".len()..];
        let input = after_input.lines().next().unwrap_or("{}").trim().to_string();

        Some((action, input))
    }

    fn truncate(&self, s: &str, max: usize) -> String {
        if s.len() <= max {
            s.to_string()
        } else {
            format!("{}...\n[truncated, {} total chars]", &s[..max], s.len())
        }
    }

    /// Quick shorthand: run a task and log.
    pub async fn run_and_log(&self, task: &str) -> anyhow::Result<String> {
        let result = self.run(task).await?;
        tracing::info!(agent = %self.name, task = %task, result_len = result.len(), "Completed");
        Ok(result)
    }
}
