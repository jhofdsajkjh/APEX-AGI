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

pub mod engines;
pub mod feedback;
pub mod inference;
pub mod knowledge;
pub mod memory;
pub mod orchestrator;
pub mod tool;

// Re-export old modules for backward compatibility
pub mod llm;
pub mod react;
pub mod tools;

use feedback::{FeedbackCollector, FeedbackIntegrator};
use inference::InferenceEngine;
use memory::{LongTermMemory, MemoryConfig, MemoryType};
use std::sync::Arc;
use tool::ToolRegistry;

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
        let name_str: String = name.into();
        let memory = LongTermMemory::new(memory_config, engine.clone(), &name_str);
        Self {
            engine,
            tools,
            memory,
            feedback_collector: None,
            feedback_integrator: None,
            name: name_str,
            max_steps: 15,
        }
    }

    /// Create from environment (backward-compatible).
    pub fn from_env(tools: ToolRegistry) -> Self {
        let engine: Arc<dyn InferenceEngine> = if std::env::var("OMEGA_API_KEY")
            .ok()
            .and_then(|k| if k.is_empty() { None } else { Some(k) })
            .is_some()
        {
            Arc::new(engines::OpenAIEngine::from_env())
        } else {
            tracing::warn!(
                "OMEGA_API_KEY not set — using MockEngine. Set in .env for real LLM access."
            );
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
                self.memory
                    .store_tagged(
                        format!("Task completed: {}\nResult: {}", task, answer),
                        MemoryType::Procedural,
                        0.8,
                        vec!["success".into(), self.name.clone()],
                    )
                    .await;
                return Ok(answer);
            }

            // Try Action
            if let Some((action, args_json)) = self.parse_action(&content) {
                let args: serde_json::Value = serde_json::from_str(&args_json)
                    .unwrap_or(serde_json::Value::Object(Default::default()));

                tracing::debug!(step, action = %action, args = %args_json, "Executing tool");

                let result = self.tools.execute(&action, &args).await;
                let obs = if result.success {
                    format!("Observation:\n{}", self.truncate(&result.output, 4000))
                } else {
                    format!(
                        "Observation (error):\n{}",
                        self.truncate(&result.output, 2000)
                    )
                };

                tracing::debug!(step, tool_success = result.success, "Tool result");
                messages.push(inference::Message::user(&obs));

                // Store in memory
                self.memory
                    .store_tagged(
                        format!(
                            "Step {}: Action {} → {}",
                            step,
                            action,
                            if result.success { "success" } else { "failed" }
                        ),
                        MemoryType::Episodic,
                        0.4,
                        vec![action, self.name.clone()],
                    )
                    .await;
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
             Please provide your final answer now based on what you've learned so far.",
        ));
        let final_resp = self.engine.chat(&messages).await?;
        Ok(self.parse_answer(&final_resp.content).unwrap_or_else(|| {
            format!(
                "Maximum steps reached ({}). Last response:\n{}",
                self.max_steps, final_resp.content
            )
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
        // === Strategy 1: Exact "Answer:" prefix (existing behavior) ===
        if let Some(answer) = self.parse_answer_exact(text, "Answer:") {
            return Some(answer);
        }

        // === Strategy 2: Broader case variations via manual search ===
        for prefix in &[
            "Answer:",
            "answer:",
            "ANSWER:",
            "Final Answer:",
            "final answer:",
            "FINAL ANSWER:",
            "Final answer:",
        ] {
            if let Some(answer) = self.parse_answer_exact(text, prefix) {
                return Some(answer);
            }
        }

        // === Strategy 3: Markdown format extraction ===
        // Look for **Answer** pattern (bold in markdown)
        if let Some(start) = text.find("**Answer**") {
            let after = &text[start + "**Answer**".len()..];
            let after = after.trim_start_matches(':').trim();
            let end = after
                .find("\n**")
                .or_else(|| after.find("\n\n"))
                .unwrap_or(after.len());
            let candidate = after[..end].trim();
            if !candidate.is_empty() {
                return Some(candidate.to_string());
            }
        }
        if let Some(start) = text.find("**answer**") {
            let after = &text[start + "**answer**".len()..];
            let after = after.trim_start_matches(':').trim();
            let end = after
                .find("\n**")
                .or_else(|| after.find("\n\n"))
                .unwrap_or(after.len());
            let candidate = after[..end].trim();
            if !candidate.is_empty() {
                return Some(candidate.to_string());
            }
        }

        // Look for blockquote > Answer pattern
        for line in text.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("> **Answer**") || trimmed.starts_with("> **answer**") {
                let after = trimmed.trim_start_matches('>').trim();
                let answer_text = after.trim_start_matches("**Answer**").trim();
                let answer_text = answer_text.trim_start_matches("**answer**").trim();
                let answer_text = answer_text.trim_start_matches(':').trim();
                if !answer_text.is_empty() {
                    return Some(answer_text.to_string());
                }
            }
        }

        // === Strategy 4: Last non-empty paragraph as fallback ===
        let paragraphs: Vec<&str> = text
            .split("\n\n")
            .map(|s| s.trim())
            .filter(|s| !s.is_empty() && s.len() > 20)
            .collect();
        if let Some(last) = paragraphs.last() {
            // Only use last paragraph if it looks like an answer (not an action or thought)
            let lower = last.to_lowercase();
            if !lower.starts_with("thought:")
                && !lower.starts_with("action:")
                && !lower.starts_with("tool:")
            {
                return Some(last.to_string());
            }
        }

        None
    }

    /// Helper: find a prefix and extract text after it, stopping at Action:\n
    fn parse_answer_exact<'a>(&self, text: &'a str, prefix: &str) -> Option<String> {
        let idx = text.find(prefix)?;
        let after = &text[idx + prefix.len()..];
        // Truncate at the next Action: line or double newline
        let end = after.find("\nAction:").unwrap_or(after.len());
        let end = after[..end].find("\n\n").unwrap_or(end);
        let result = after[..end].trim().to_string();
        if result.is_empty() {
            None
        } else {
            Some(result)
        }
    }

    fn parse_action(&self, text: &str) -> Option<(String, String)> {
        // Find the action line: look for "Action:" or "Tool:" (alternative format)
        let action_trigger = if let Some(pos) = text.find("Action:") {
            (pos, "Action:")
        } else if let Some(pos) = text.find("Tool:") {
            (pos, "Tool:")
        } else {
            return None;
        };

        let (action_idx, trigger) = action_trigger;
        let after_action = &text[action_idx + trigger.len()..];
        let action_line = after_action.lines().next()?.trim();
        let action = action_line.split_whitespace().next()?.to_lowercase();

        // Find Action Input: — search from after the action line onward
        let remaining_after_action = &text[action_idx + trigger.len() + action_line.len()..];
        let input_text = if let Some(input_idx) = remaining_after_action.find("Action Input:") {
            let after_input = &remaining_after_action[input_idx + "Action Input:".len()..];

            // Try same-line JSON first: trim and check if it starts with '{' or '['
            let first_line = after_input
                .lines()
                .next()
                .unwrap_or("{}")
                .trim()
                .to_string();
            if first_line.starts_with('{') || first_line.starts_with('[') {
                // Same-line JSON — collect until the closing brace/bracket or next section
                let mut depth: i32 = 0;
                let mut json_buf = String::new();
                let mut started = false;
                for ch in first_line.chars() {
                    match ch {
                        '{' | '[' => {
                            depth += 1;
                            started = true;
                            json_buf.push(ch);
                        }
                        '}' | ']' => {
                            depth -= 1;
                            json_buf.push(ch);
                            if started && depth <= 0 {
                                break;
                            }
                        }
                        _ => {
                            if started {
                                json_buf.push(ch);
                            }
                        }
                    }
                }
                if !json_buf.is_empty() && depth <= 0 {
                    json_buf
                } else {
                    first_line
                }
            } else {
                // JSON on next line(s) — collect subsequent lines until blank line or next section
                let mut lines = after_input.lines();
                lines.next(); // skip the (already captured) first line
                let mut json_buf = first_line; // may contain partial JSON
                let mut depth: i32 = 0;
                for ch in json_buf.chars() {
                    match ch {
                        '{' | '[' => depth += 1,
                        '}' | ']' => depth -= 1,
                        _ => {}
                    }
                }
                for line in lines {
                    let trimmed = line.trim();
                    if trimmed.is_empty()
                        || trimmed.starts_with("Thought:")
                        || trimmed.starts_with("Action:")
                        || trimmed.starts_with("Tool:")
                        || trimmed.starts_with("Answer:")
                    {
                        break;
                    }
                    for ch in trimmed.chars() {
                        match ch {
                            '{' | '[' => depth += 1,
                            '}' | ']' => depth -= 1,
                            _ => {}
                        }
                    }
                    json_buf.push_str(trimmed);
                    if depth <= 0 {
                        break;
                    }
                }
                json_buf
            }
        } else {
            // No explicit "Action Input:" — try to grab JSON from the action line itself
            let parts: Vec<&str> = action_line.split_whitespace().collect();
            if parts.len() > 1 {
                parts[1..].join(" ")
            } else {
                "{}".to_string()
            }
        };

        Some((action, input_text))
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
