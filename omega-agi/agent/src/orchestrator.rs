//! # Multi-Agent Orchestration System
//!
//! Manages multiple specialized agents that work together to accomplish complex tasks.
//!
//! ```text
//! ┌─────────────────────────────────────────┐
//! │         Orchestrator Agent              │
//! │  (task decomposition, scheduling,       │
//! │   conflict resolution, result merging)  │
//! ├────────┬────────┬────────┬──────────────┤
//! │ Coding │Research│ System │   Monitor    │
//! │ Agent  │ Agent  │ Agent  │   Agent      │
//! │写代码  │搜资料  │运维操作│  监控告警    │
//! └────────┴────────┴────────┴──────────────┘
//! ```
//!
//! Each agent has its own:
//! - Role description (system prompt)
//! - Tool set (permissions)
//! - Memory (conversation history + long-term knowledge)
//! - Inference engine (can be different per agent)

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};

use crate::feedback::FeedbackCollector;
use crate::inference::{self, InferenceEngine, Message};
use crate::memory::{LongTermMemory, MemoryConfig, MemoryType};
use crate::tool::{ToolRegistry, ToolResult};

// ---------------------------------------------------------------------------
// Agent Identity
// ---------------------------------------------------------------------------

/// Defines an agent's role and capabilities.
#[derive(Debug, Clone)]
pub struct AgentIdentity {
    /// Unique name (e.g., "coding-agent", "research-agent")
    pub name: String,
    /// Role description for system prompt
    pub role: String,
    /// Allowed tool names (empty = all tools)
    pub allowed_tools: Vec<String>,
    /// Inference engine config overrides
    pub model: String,
    pub temperature: f32,
}

/// Pre-defined agent roles.
impl AgentIdentity {
    /// Orchestrator — decomposes tasks and coordinates specialists.
    pub fn orchestrator() -> Self {
        Self {
            name: "orchestrator".into(),
            role: "You are the Orchestrator Agent. Your job is to:\n\
                   1. Understand the user's overall goal\n\
                   2. Break it down into sub-tasks\n\
                   3. Assign each sub-task to the appropriate specialist agent\n\
                   4. Review and merge results\n\
                   5. Ensure consistency across all work"
                .into(),
            allowed_tools: vec![
                "assign_task".into(),
                "review_result".into(),
                "finalize".into(),
            ],
            model: "gpt-4o".into(),
            temperature: 0.3,
        }
    }

    /// Coding agent — writes and modifies code.
    pub fn coding() -> Self {
        Self {
            name: "coding-agent".into(),
            role: "You are the Coding Agent. You write high-quality, well-tested Rust code.\n\
                   - Read existing code before modifying\n\
                   - Write thorough tests\n\
                   - Follow Rust best practices\n\
                   - Generate documentation"
                .into(),
            allowed_tools: vec![
                "read".into(),
                "write".into(),
                "search".into(),
                "bash".into(),
                "codegen".into(),
            ],
            model: "gpt-4o".into(),
            temperature: 0.2,
        }
    }

    /// Research agent — searches for information.
    pub fn research() -> Self {
        Self {
            name: "research-agent".into(),
            role: "You are the Research Agent. You find information and analyze code.\n\
                   - Search through codebase\n\
                   - Analyze code patterns\n\
                   - Identify performance bottlenecks\n\
                   - Provide detailed reports"
                .into(),
            allowed_tools: vec![
                "read".into(),
                "search".into(),
                "health".into(),
                "diagnose".into(),
            ],
            model: "gpt-4o".into(),
            temperature: 0.4,
        }
    }

    /// System agent — manages infrastructure.
    pub fn system_agent() -> Self {
        Self {
            name: "system-agent".into(),
            role: "You are the System Agent. You manage infrastructure and operations.\n\
                   - Run health checks\n\
                   - Execute system commands\n\
                   - Monitor system resources\n\
                   - Handle deployments"
                .into(),
            allowed_tools: vec![
                "bash".into(),
                "health".into(),
                "diagnose".into(),
                "heal".into(),
            ],
            model: "gpt-4o".into(),
            temperature: 0.2,
        }
    }

    /// Monitor agent — watches for issues.
    pub fn monitor() -> Self {
        Self {
            name: "monitor-agent".into(),
            role: "You are the Monitor Agent. You watch the system for anomalies.\n\
                   - Continuously check system health\n\
                   - Detect performance regressions\n\
                   - Alert on errors\n\
                   - Suggest proactive fixes"
                .into(),
            allowed_tools: vec!["health".into(), "diagnose".into()],
            model: "gpt-4o".into(),
            temperature: 0.1,
        }
    }
}

// ---------------------------------------------------------------------------
// SpecialistAgent — a single agent instance
// ---------------------------------------------------------------------------

/// A single specialist agent with its own identity, tools, and memory.
pub struct SpecialistAgent {
    pub identity: AgentIdentity,
    engine: Box<dyn InferenceEngine>,
    tools: ToolRegistry,
    memory: LongTermMemory,
    feedback: Option<Arc<FeedbackCollector>>,
    message_log: RwLock<Vec<Message>>,
}

impl SpecialistAgent {
    /// Create a new specialist agent.
    pub fn new(
        identity: AgentIdentity,
        engine: Box<dyn InferenceEngine>,
        tools: ToolRegistry,
        memory: LongTermMemory,
    ) -> Self {
        Self {
            identity,
            engine,
            tools,
            memory,
            feedback: None,
            message_log: RwLock::new(Vec::new()),
        }
    }

    /// Attach a feedback collector.
    pub fn with_feedback(mut self, feedback: Arc<FeedbackCollector>) -> Self {
        self.feedback = Some(feedback);
        self
    }

    /// Run a task with this agent.
    pub async fn run(&self, task: &str) -> anyhow::Result<String> {
        let mut messages = vec![
            Message::system(self.build_system_prompt()),
            Message::user(task),
        ];

        let max_steps = 15;
        for step in 0..max_steps {
            let resp = self.engine.chat(&messages).await?;
            let content = resp.content.clone();
            messages.push(Message::assistant(&content));

            // Log the message
            self.message_log
                .write()
                .await
                .push(Message::assistant(&content));

            // Try to parse Answer/Action
            if let Some(answer) = self.parse_answer(&content) {
                // Store in memory
                self.memory
                    .store_tagged(
                        format!("Task: {}\nResult: {}", task, answer),
                        MemoryType::Procedural,
                        0.7,
                        vec![self.identity.name.clone(), "task-complete".into()],
                    )
                    .await;
                return Ok(answer);
            }

            if let Some((action, args)) = self.parse_action(&content) {
                // Check permissions
                if !self.identity.allowed_tools.is_empty()
                    && !self.identity.allowed_tools.contains(&action)
                {
                    messages.push(Message::user(format!(
                        "Error: You don't have permission to use '{}'. Allowed: {}",
                        action,
                        self.identity.allowed_tools.join(", ")
                    )));
                    continue;
                }

                let result = self
                    .tools
                    .execute(&action, &serde_json::from_str(&args).unwrap_or_default())
                    .await;
                let obs = if result.success {
                    format!("Observation:\n{}", result.output)
                } else {
                    format!("Observation (error):\n{}", result.output)
                };

                messages.push(Message::user(&obs));

                // Store in memory
                self.memory
                    .store_tagged(
                        format!("Action: {}({}) → {}", action, args, result.output),
                        MemoryType::Episodic,
                        0.5,
                        vec![self.identity.name.clone(), action],
                    )
                    .await;
            } else {
                // Format error — help the LLM
                messages.push(Message::user(
                    "Your response was not in the correct format. Please respond with:\n\
                     Thought: <reasoning>\nAction: <tool_name>\nAction Input: <JSON args>\n\n\
                     Or:\nThought: <reasoning>\nAnswer: <final answer>",
                ));
            }
        }

        // Max steps reached — force final answer
        messages.push(Message::user(
            "Maximum steps reached. Please provide your final answer now.",
        ));
        let final_resp = self.engine.chat(&messages).await?;
        let answer = self
            .parse_answer(&final_resp.content)
            .unwrap_or(final_resp.content);
        Ok(answer)
    }

    fn build_system_prompt(&self) -> String {
        format!(
            "{}\n\n{}\n\nYou have access to these tools: {}\n\n\
             Always respond with:\nThought: <reasoning>\nAction: <tool>\nAction Input: <json>\n\
             or:\nThought: <reasoning>\nAnswer: <final answer>",
            self.identity.role,
            self.tools.system_prompt(),
            if self.identity.allowed_tools.is_empty() {
                "all tools".into()
            } else {
                self.identity.allowed_tools.join(", ")
            }
        )
    }

    fn parse_answer(&self, text: &str) -> Option<String> {
        let idx = text.find("Answer:")?;
        let after = &text[idx + "Answer:".len()..];
        // Stop at next section or end
        let end = after.find("\n\n").unwrap_or(after.len());
        Some(after[..end].trim().to_string())
    }

    fn parse_action(&self, text: &str) -> Option<(String, String)> {
        let action_idx = text.find("Action:")?;
        let after_action = &text[action_idx + "Action:".len()..];
        let action = after_action
            .lines()
            .next()?
            .trim()
            .to_lowercase()
            .to_string();

        let input_idx = text.find("Action Input:")?;
        let after_input = &text[input_idx + "Action Input:".len()..];
        let input = after_input
            .lines()
            .next()
            .unwrap_or("{}")
            .trim()
            .to_string();

        Some((action, input))
    }
}

// ---------------------------------------------------------------------------
// Task — a unit of work for the multi-agent system
// ---------------------------------------------------------------------------

/// A task that an agent should execute.
#[derive(Debug, Clone)]
pub struct Task {
    pub id: String,
    pub description: String,
    pub assigned_to: String,
    pub context: HashMap<String, String>,
    pub dependencies: Vec<String>,
    pub status: TaskStatus,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TaskStatus {
    Pending,
    InProgress,
    Completed(String),
    Failed(String),
}

// ---------------------------------------------------------------------------
// OrchestratorAgent — coordinates multiple specialists
// ---------------------------------------------------------------------------

/// The orchestrator that decomposes tasks and manages specialist agents.
pub struct Orchestrator {
    /// The orchestrator's own engine.
    engine: Box<dyn InferenceEngine>,
    /// Specialist agents indexed by name.
    specialists: HashMap<String, SpecialistAgent>,
    /// Task queue.
    tasks: RwLock<Vec<Task>>,
    /// Current session ID.
    session_id: String,
    /// Feedback collector (shared).
    feedback: Option<Arc<FeedbackCollector>>,
}

impl Orchestrator {
    /// Create a new orchestrator with the given inference engine.
    pub fn new(engine: Box<dyn InferenceEngine>) -> Self {
        Self {
            engine,
            specialists: HashMap::new(),
            tasks: RwLock::new(Vec::new()),
            session_id: format!("session-{}", chrono::Utc::now().timestamp_nanos()),
            feedback: None,
        }
    }

    /// Attach a feedback collector.
    pub fn with_feedback(mut self, feedback: Arc<FeedbackCollector>) -> Self {
        self.feedback = Some(feedback);
        self
    }

    /// Register a specialist agent.
    pub fn register(&mut self, agent: SpecialistAgent) {
        self.specialists.insert(agent.identity.name.clone(), agent);
    }

    /// Register multiple specialist agents.
    pub fn register_all(&mut self, agents: Vec<SpecialistAgent>) {
        for agent in agents {
            self.register(agent);
        }
    }

    /// Create and register default specialist agents.
    pub fn with_default_specialists(
        mut self,
        engine_factory: impl Fn(&str) -> Box<dyn InferenceEngine>,
        base_tools: ToolRegistry,
        memory_config: MemoryConfig,
        feedback: Option<Arc<FeedbackCollector>>,
    ) -> Self {
        let identities = vec![
            AgentIdentity::coding(),
            AgentIdentity::research(),
            AgentIdentity::system_agent(),
            AgentIdentity::monitor(),
        ];

        for identity in identities {
            let engine = engine_factory(&identity.name);
            let memory = LongTermMemory::new(
                memory_config.clone(),
                Arc::from(engine.box_clone()),
                &identity.name,
            );
            let mut agent = SpecialistAgent::new(
                identity,
                engine,
                base_tools.clone(), // Clone registry — each agent has its own copy
                memory,
            );
            if let Some(ref fb) = feedback {
                agent = agent.with_feedback(fb.clone());
            }
            self.register(agent);
        }

        self
    }

    /// Decompose a complex task into sub-tasks.
    pub async fn decompose(&self, task: &str) -> anyhow::Result<Vec<Task>> {
        let prompt = format!(
            "Decompose the following task into 2-4 smaller, specific sub-tasks that can be \
             executed independently by specialist agents. Available agents: {}\n\n\
             Task: {}\n\n\
             For each sub-task, specify:\n\
             1. A clear description\n\
             2. Which agent should handle it\n\
             3. Any dependencies on other sub-tasks\n\n\
             Format as a JSON array where each entry has: description, assigned_to, dependencies.\n\
             Do NOT use markdown code blocks — output pure JSON only.",
            self.specialists
                .keys()
                .cloned()
                .collect::<Vec<_>>()
                .join(", "),
            task
        );

        let resp = self.engine.chat(&[Message::user(&prompt)]).await?;

        // Parse JSON response
        let json_str = resp
            .content
            .trim()
            .trim_start_matches("```json")
            .trim_start_matches("```")
            .trim_end_matches("```")
            .trim();

        let tasks_data: Vec<serde_json::Value> = match serde_json::from_str(json_str) {
            Ok(v) => v,
            Err(_) => {
                // Fallback: create a single task for the coding agent
                return Ok(vec![Task {
                    id: format!("task-{}", chrono::Utc::now().timestamp_nanos()),
                    description: task.to_string(),
                    assigned_to: "coding-agent".into(),
                    context: HashMap::new(),
                    dependencies: vec![],
                    status: TaskStatus::Pending,
                }]);
            }
        };

        let tasks: Vec<Task> = tasks_data
            .into_iter()
            .map(|t| {
                let desc = t["description"].as_str().unwrap_or(task);
                let agent = t["assigned_to"].as_str().unwrap_or("coding-agent");
                let deps: Vec<String> = t["dependencies"]
                    .as_array()
                    .map(|a| {
                        a.iter()
                            .filter_map(|v| v.as_str().map(|s| s.to_string()))
                            .collect()
                    })
                    .unwrap_or_default();

                // Find the best agent if specified agent doesn't exist
                let assigned = if self.specialists.contains_key(agent) {
                    agent.to_string()
                } else {
                    // Fallback: try to find a suitable agent
                    let lower_desc = desc.to_lowercase();
                    if lower_desc.contains("code")
                        || lower_desc.contains("implement")
                        || lower_desc.contains("write")
                    {
                        "coding-agent".to_string()
                    } else if lower_desc.contains("search")
                        || lower_desc.contains("research")
                        || lower_desc.contains("find")
                    {
                        "research-agent".to_string()
                    } else if lower_desc.contains("system")
                        || lower_desc.contains("deploy")
                        || lower_desc.contains("infra")
                    {
                        "system-agent".to_string()
                    } else if lower_desc.contains("monitor")
                        || lower_desc.contains("watch")
                        || lower_desc.contains("alert")
                    {
                        "monitoring-agent".to_string()
                    } else {
                        "coding-agent".to_string()
                    }
                };

                Task {
                    id: format!("task-{}", chrono::Utc::now().timestamp_nanos()),
                    description: desc.to_string(),
                    assigned_to: assigned,
                    context: HashMap::new(),
                    dependencies: deps,
                    status: TaskStatus::Pending,
                }
            })
            .collect();

        Ok(tasks)
    }

    /// Execute a complex task through multi-agent collaboration.
    pub async fn execute(&self, task: &str) -> anyhow::Result<String> {
        tracing::info!("[Orchestrator] Decomposing task: {}", task);

        let sub_tasks = self.decompose(task).await?;
        let mut results: HashMap<String, String> = HashMap::new();

        tracing::info!(
            "[Orchestrator] Decomposed into {} sub-tasks",
            sub_tasks.len()
        );

        // Execute sub-tasks respecting dependencies
        let mut remaining: Vec<Task> = sub_tasks;
        let mut completed = Vec::new();

        while !remaining.is_empty() {
            let mut progress = false;

            for i in (0..remaining.len()).rev() {
                let task = &remaining[i];

                // Check dependencies
                let deps_met = task.dependencies.iter().all(|dep| {
                    results.contains_key(dep)
                        || completed.iter().any(|c: &Task| {
                            c.description.contains(dep)
                                && matches!(c.status, TaskStatus::Completed(_))
                        })
                });

                if !deps_met {
                    continue;
                }

                // Execute this task
                let mut agent_task = remaining.remove(i);
                agent_task.status = TaskStatus::InProgress;

                tracing::info!(
                    "[Orchestrator] Assigning to {}: {}",
                    agent_task.assigned_to,
                    &agent_task.description
                );

                let result = match self.specialists.get(&agent_task.assigned_to) {
                    Some(agent) => {
                        let enriched_task = format!(
                            "Context from previous steps:\n{}\n\nTask: {}",
                            results
                                .values()
                                .cloned()
                                .collect::<Vec<_>>()
                                .join("\n---\n"),
                            agent_task.description
                        );
                        agent.run(&enriched_task).await
                    }
                    None => Err(anyhow::anyhow!(
                        "Unknown specialist: {}",
                        agent_task.assigned_to
                    )),
                };

                match result {
                    Ok(output) => {
                        agent_task.status = TaskStatus::Completed(output.clone());
                        results.insert(agent_task.id.clone(), output);
                        progress = true;
                    }
                    Err(e) => {
                        agent_task.status = TaskStatus::Failed(e.to_string());
                        results.insert(agent_task.id.clone(), format!("Error: {}", e));
                        progress = true;
                    }
                }

                completed.push(agent_task);
            }

            if !progress {
                // Deadlock or no runnable tasks — force schedule the first one
                if let Some(mut task) = remaining.pop() {
                    tracing::warn!(
                        "[Orchestrator] Force-executing task (possible deadlock): {}",
                        task.description
                    );
                    task.status = TaskStatus::InProgress;

                    let result = match self.specialists.get(&task.assigned_to) {
                        Some(agent) => agent.run(&task.description).await,
                        None => Err(anyhow::anyhow!("Unknown specialist: {}", task.assigned_to)),
                    };

                    match result {
                        Ok(output) => {
                            task.status = TaskStatus::Completed(output.clone());
                            results.insert(task.id.clone(), output);
                        }
                        Err(e) => {
                            task.status = TaskStatus::Failed(e.to_string());
                            results.insert(task.id.clone(), format!("Error: {}", e));
                        }
                    }
                    completed.push(task);
                } else {
                    break;
                }
            }
        }

        // Synthesize final result
        let final_prompt = format!(
            "Synthesize the following results from multiple agents into a coherent final answer.\n\n\
             Original task: {}\n\n\
             Results:\n{}\n\n\
             Provide a unified, well-structured answer.",
            task,
            completed.iter().map(|t| {
                match &t.status {
                    TaskStatus::Completed(r) => format!("## {} ({})\n{}\n", t.assigned_to, t.description, r),
                    TaskStatus::Failed(e) => format!("## {} ({})\n**Failed:** {}\n", t.assigned_to, t.description, e),
                    _ => String::new(),
                }
            }).collect::<Vec<_>>().join("\n")
        );

        let final_resp = self.engine.chat(&[Message::user(&final_prompt)]).await?;

        // Record feedback
        if let Some(ref fb) = self.feedback {
            let success_rate = completed
                .iter()
                .filter(|t| matches!(t.status, TaskStatus::Completed(_)))
                .count() as f64
                / completed.len().max(1) as f64;
            fb.record_quality(
                success_rate,
                &format!("Multi-agent task completed: {}", task),
            )
            .await;
        }

        Ok(final_resp.content)
    }

    /// Get status of all agents.
    pub fn agent_status(&self) -> Vec<String> {
        self.specialists.keys().cloned().collect()
    }

    /// Get task status.
    pub async fn task_status(&self) -> Vec<(String, String)> {
        let tasks = self.tasks.read().await;
        tasks
            .iter()
            .map(|t| (t.description.clone(), format!("{:?}", t.status)))
            .collect()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engines::MockEngine;
    use crate::tool::ThinkTool;

    #[tokio::test]
    async fn test_specialist_agent_creation() {
        let engine = Box::new(MockEngine::new());
        let mut tools = ToolRegistry::new();
        tools.register(Box::new(ThinkTool));

        let memory = LongTermMemory::new(
            MemoryConfig {
                db_path: String::new(),
                ..Default::default()
            },
            Arc::new(MockEngine::new()),
            "test-agent",
        );

        let agent = SpecialistAgent::new(AgentIdentity::coding(), engine, tools, memory);

        assert_eq!(agent.identity.name, "coding-agent");
    }

    #[test]
    fn test_orchestrator_creation() {
        let engine = Box::new(MockEngine::new());
        let orchestrator = Orchestrator::new(engine);
        assert!(orchestrator.agent_status().is_empty());
    }

    #[test]
    fn test_register_specialist() {
        let engine = Box::new(MockEngine::new());
        let mut orchestrator = Orchestrator::new(engine);

        let specialist_engine = Box::new(MockEngine::new());
        let tools = ToolRegistry::new();
        let memory = LongTermMemory::new(
            MemoryConfig {
                db_path: String::new(),
                ..Default::default()
            },
            Arc::new(MockEngine::new()),
            "coding-agent",
        );

        orchestrator.register(SpecialistAgent::new(
            AgentIdentity::coding(),
            specialist_engine,
            tools,
            memory,
        ));

        assert_eq!(orchestrator.agent_status().len(), 1);
        assert!(orchestrator
            .agent_status()
            .contains(&"coding-agent".to_string()));
    }

    #[test]
    fn test_agent_identity_creation() {
        let identities = vec![
            AgentIdentity::orchestrator(),
            AgentIdentity::coding(),
            AgentIdentity::research(),
            AgentIdentity::system_agent(),
            AgentIdentity::monitor(),
        ];
        assert_eq!(identities.len(), 5);
        for id in &identities {
            assert!(!id.name.is_empty());
            assert!(!id.role.is_empty());
        }
    }
}
