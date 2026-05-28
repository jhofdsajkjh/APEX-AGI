//! ReAct (Reasoning + Acting) loop engine — updated to use InferenceEngine trait.
//!
//! Cycles: Thought → Action → Observation → Thought → … → Answer.
//! The LLM decides when it has enough information to produce the final answer.

use crate::inference::{InferenceEngine, Message};
use crate::tool::{ToolContext, ToolResult};
use tracing::{debug, info, warn};

// ---------------------------------------------------------------------------
// Parsed action from the LLM
// ---------------------------------------------------------------------------

enum LlmStep {
    Action { name: String, args: serde_json::Value },
    Answer(String),
    ParseFailed(String),
}

/// Extract the first `Action:` / `Answer:` block from the LLM output.
fn parse_step(text: &str) -> LlmStep {
    // Try Answer first (prefer final answer over intermediate action)
    if let Some(ans) = extract_tag(text, "Answer:", None) {
        return LlmStep::Answer(ans.trim().to_string());
    }

    // Try Action + Action Input
    if let Some(name) = extract_tag(text, "Action:", None) {
        let args_raw = extract_tag(text, "Action Input:", Some("Action:")).unwrap_or("{}");
        let args: serde_json::Value =
            serde_json::from_str(args_raw.trim()).unwrap_or(serde_json::Value::Object(Default::default()));
        return LlmStep::Action {
            name: name.trim().to_lowercase(),
            args,
        };
    }

    LlmStep::ParseFailed(format!(
        "Could not parse LLM response into Action/Answer format.\nRaw output:\n{}",
        text
    ))
}

fn extract_tag<'a>(text: &'a str, tag: &str, stop_tag: Option<&str>) -> Option<&'a str> {
    let idx = text.find(tag)?;
    let start = idx + tag.len();
    let text_after = &text[start..];

    let end = stop_tag
        .and_then(|stop| text_after.find(stop))
        .unwrap_or(text_after.len());

    Some(text_after[..end].trim())
}

// ---------------------------------------------------------------------------
// System prompt builder
// ---------------------------------------------------------------------------

fn build_system_prompt() -> String {
    format!(
        r#"You are **OMEGA AGI Agent** — the autonomous reasoning and execution layer
of a five-layer AGI system (HyperCore → Runtime → Engineering → Evolution → Adapters).

Your role: understand the user's task, reason step by step, and use the available
tools to accomplish it. You can also use `think` as a tool to do internal reasoning
without calling any external function — just write your reasoning as the "output".

{}

Important rules:
1. Always start with `Thought:` explaining your reasoning.
2. Then use exactly one `Action:` + `Action Input:` per turn.
3. Wait for the `Observation:` result before thinking again.
4. When you have enough information, produce `Answer:` with the final response.
5. If a tool fails, try an alternative approach or explain the error.
6. Be thorough — check system health first if the task is unclear.
7. You operate inside the OMEGA AGI project at C:\Users\Fogtao\Downloads\omega-agi-supremacy.
"#,
        tool_descriptions_prompt()
    )
}

// ---------------------------------------------------------------------------
// ReAct engine
// ---------------------------------------------------------------------------

pub struct ReActEngine {
    max_steps: usize,
}

impl Default for ReActEngine {
    fn default() -> Self {
        Self { max_steps: 15 }
    }
}

impl ReActEngine {
    pub fn new(max_steps: usize) -> Self {
        Self { max_steps }
    }

    /// Run the ReAct loop using a generic InferenceEngine.
    pub async fn run(
        &self,
        llm: &dyn InferenceEngine,
        context: &dyn ToolContext,
        task: &str,
    ) -> anyhow::Result<String> {
        let system_prompt = build_system_prompt();
        let mut messages = vec![
            Message::system(system_prompt),
            Message::user(task.to_string()),
        ];

        for step in 0..self.max_steps {
            debug!(step, "ReAct iteration");

            let reply = llm.chat(&messages).await?;
            let content = reply.content.clone();

            debug!(step, response_len = content.len(), "LLM response");
            messages.push(Message::assistant(&content));

            match parse_step(&content) {
                LlmStep::Answer(answer) => {
                    info!(step, "Got final answer");
                    return Ok(answer);
                }

                LlmStep::Action { name, args } => {
                    debug!(step, action = %name, "Executing tool");

                    match name.as_str() {
                        "think" | "reason" => {
                            // Internal thought — just log and continue
                            let thought = args.get("thought")
                                .and_then(|v| v.as_str())
                                .unwrap_or("...");
                            debug!("Internal thought: {}", thought);
                            messages.push(Message::user(format!(
                                "Observation (internal thought): {}",
                                thought
                            )));
                            continue;
                        }
                        _ => {}
                    }

                    // Execute the tool
                    let result = context.execute_tool(&name, &args).await;
                    let obs = if result.success {
                        format!("Observation:\n{}", result.output)
                    } else {
                        format!("Observation (error):\n{}", result.output)
                    };

                    debug!(step, observation = %obs, "Tool result");
                    messages.push(Message::user(obs));
                }

                LlmStep::ParseFailed(msg) => {
                    warn!(step, "Parse failure: {}", msg);
                    messages.push(Message::user(format!(
                        "Your response was not in the correct format. Please respond with:\n\
                         Thought: <reasoning>\nAction: <tool_name>\nAction Input: <JSON args>\n\n\
                         Or if you have the final answer:\nThought: <reasoning>\nAnswer: <final answer>"
                    )));
                }
            }
        }

        // If we exhaust steps without an Answer, ask the LLM to summarise
        messages.push(Message::user(
            "You have reached the maximum number of reasoning steps. \
             Please provide your final answer now based on what you've learned so far.",
        ));
        let final_reply = llm.chat(&messages).await?;
        // Try to extract Answer one more time
        match parse_step(&final_reply.content) {
            LlmStep::Answer(a) => Ok(a),
            _ => Ok(format!(
                "Reached max steps ({}) without a final answer. Last LLM output:\n{}",
                self.max_steps, final_reply.content
            )),
        }
    }
}

// Re-export from tools module for backward compatibility
pub use crate::tools::{tool_descriptions_prompt, ToolContext, ToolResult};
