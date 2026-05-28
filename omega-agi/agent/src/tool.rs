//! # Tool Trait — Tool-as-object pattern
//!
//! Replaces the old single-match dispatch with a proper trait-based system.
//! Each tool is a self-contained object that knows:
//! - Its name and description
//! - Its JSON schema for arguments
//! - How to execute
//!
//! New tools can be added without modifying any existing code.

use async_trait::async_trait;
use serde::Serialize;
use std::collections::HashMap;
use std::sync::Arc;

// ---------------------------------------------------------------------------
// Core types
// ---------------------------------------------------------------------------

/// The result of a single tool execution.
#[derive(Debug, Clone)]
pub struct ToolResult {
    pub success: bool,
    pub output: String,
    pub tool_name: String,
}

impl ToolResult {
    pub fn ok(tool_name: impl Into<String>, output: impl Into<String>) -> Self {
        Self { success: true, tool_name: tool_name.into(), output: output.into() }
    }
    pub fn err(tool_name: impl Into<String>, msg: impl Into<String>) -> Self {
        Self { success: false, tool_name: tool_name.into(), output: msg.into() }
    }
}

/// JSON schema for tool arguments (OpenAI function-calling format).
#[derive(Debug, Clone, Serialize)]
pub struct ToolSchema {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}

// ---------------------------------------------------------------------------
// Tool trait
// ---------------------------------------------------------------------------

/// A single tool that the Agent can invoke.
///
/// Implement this trait to add a new tool. The tool is auto-discovered via the
/// `ToolRegistry` — no need to modify central dispatch code.
#[async_trait]
pub trait Tool: Send + Sync {
    /// Unique name (used by the LLM to call this tool).
    fn name(&self) -> &str;

    /// Human-readable description for the LLM system prompt.
    fn description(&self) -> &str;

    /// JSON schema for the arguments (OpenAI function-calling format).
    fn parameters(&self) -> serde_json::Value;

    /// Execute the tool with the given JSON arguments.
    async fn execute(&self, args: &serde_json::Value) -> ToolResult;

    /// Clone into a box (for trait-object cloning).
    fn box_clone(&self) -> Box<dyn Tool>;
}

// ---------------------------------------------------------------------------
// ToolRegistry
// ---------------------------------------------------------------------------

/// A registry that holds all available tools and dispatches calls.
pub struct ToolRegistry {
    tools: HashMap<String, Box<dyn Tool>>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self { tools: HashMap::new() }
    }

    /// Register a tool (takes ownership).
    pub fn register(&mut self, tool: Box<dyn Tool>) {
        self.tools.insert(tool.name().to_string(), tool);
    }

    /// Register multiple tools at once.
    pub fn register_all(&mut self, tools: Vec<Box<dyn Tool>>) {
        for tool in tools {
            self.register(tool);
        }
    }

    /// Execute a tool by name.
    pub async fn execute(&self, name: &str, args: &serde_json::Value) -> ToolResult {
        match self.tools.get(name) {
            Some(tool) => tool.execute(args).await,
            None => ToolResult::err(name, format!("Unknown tool: '{}'. Available: {}", name, self.available_tools().join(", "))),
        }
    }

    /// Get all registered tool names.
    pub fn available_tools(&self) -> Vec<String> {
        self.tools.keys().cloned().collect()
    }

    /// Get a tool's schema by name.
    pub fn schema(&self, name: &str) -> Option<ToolSchema> {
        self.tools.get(name).map(|t| ToolSchema {
            name: t.name().to_string(),
            description: t.description().to_string(),
            parameters: t.parameters(),
        })
    }

    /// Get all tool schemas.
    pub fn all_schemas(&self) -> Vec<ToolSchema> {
        self.tools.values().map(|t| ToolSchema {
            name: t.name().to_string(),
            description: t.description().to_string(),
            parameters: t.parameters(),
        }).collect()
    }

    /// Generate the system prompt section for all tools.
    pub fn system_prompt(&self) -> String {
        let mut out = String::from("You have access to the following tools:\n\n");
        for schema in self.all_schemas() {
            out.push_str(&format!(
                "- **{}**: {}  \n  Args: `{}`\n\n",
                schema.name,
                schema.description,
                serde_json::to_string_pretty(&schema.parameters).unwrap_or_default(),
            ));
        }
        out.push_str(
            "You MUST respond in this exact format:\n\n\
             Thought: <your reasoning>\n\
             Action: <tool_name>\n\
             Action Input: <JSON args>\n\n\
             ... or when you have the final answer:\n\n\
             Thought: <your reasoning>\n\
             Answer: <final answer>\n",
        );
        out
    }
}

impl Clone for ToolRegistry {
    fn clone(&self) -> Self {
        let mut tools = HashMap::new();
        for (k, v) in &self.tools {
            tools.insert(k.clone(), v.box_clone());
        }
        Self { tools }
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// ToolRegistry adapters — wrap old-style tools
// ---------------------------------------------------------------------------

/// Adapt an old-style `(name, execute_fn)` pair into a Tool.
pub struct FnTool {
    name: String,
    description: String,
    parameters: serde_json::Value,
    execute_fn: Arc<dyn Send + Sync + Fn(&serde_json::Value) -> ToolResult>,
}

impl FnTool {
    pub fn new(
        name: impl Into<String>,
        description: impl Into<String>,
        parameters: serde_json::Value,
        execute_fn: Arc<dyn Send + Sync + Fn(&serde_json::Value) -> ToolResult>,
    ) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            parameters,
            execute_fn,
        }
    }
}

#[async_trait]
impl Tool for FnTool {
    fn name(&self) -> &str { &self.name }
    fn description(&self) -> &str { &self.description }
    fn parameters(&self) -> serde_json::Value { self.parameters.clone() }

    async fn execute(&self, args: &serde_json::Value) -> ToolResult {
        (self.execute_fn)(args)
    }

    fn box_clone(&self) -> Box<dyn Tool> {
        Box::new(FnTool {
            name: self.name.clone(),
            description: self.description.clone(),
            parameters: self.parameters.clone(),
            execute_fn: self.execute_fn.clone(),
        })
    }
}

// ---------------------------------------------------------------------------
// Built-in tools
// ---------------------------------------------------------------------------

/// Think tool — allows the LLM to do internal reasoning.
pub struct ThinkTool;

#[async_trait]
impl Tool for ThinkTool {
    fn name(&self) -> &str { "think" }
    fn description(&self) -> &str { "Use this for internal reasoning. The output is not shown to the user." }
    fn parameters(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "thought": {"type": "string", "description": "Your internal reasoning"}
            },
            "required": ["thought"]
        })
    }

    async fn execute(&self, args: &serde_json::Value) -> ToolResult {
        let thought = args.get("thought").and_then(|v| v.as_str()).unwrap_or("...");
        ToolResult::ok("think", format!("Thought recorded ({} chars)", thought.len()))
    }

    fn box_clone(&self) -> Box<dyn Tool> { Box::new(ThinkTool) }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_tool_registry_register_and_execute() {
        let mut registry = ToolRegistry::new();
        registry.register(Box::new(ThinkTool));

        let result = registry.execute("think", &serde_json::json!({"thought": "hello"})).await;
        assert!(result.success);
    }

    #[tokio::test]
    async fn test_tool_registry_unknown_tool() {
        let registry = ToolRegistry::new();
        let result = registry.execute("nonexistent", &serde_json::json!({})).await;
        assert!(!result.success);
        assert!(result.output.contains("Unknown tool"));
    }

    #[test]
    fn test_tool_registry_available() {
        let mut registry = ToolRegistry::new();
        registry.register(Box::new(ThinkTool));
        let tools = registry.available_tools();
        assert!(tools.contains(&"think".to_string()));
    }

    #[test]
    fn test_tool_registry_system_prompt() {
        let mut registry = ToolRegistry::new();
        registry.register(Box::new(ThinkTool));
        let prompt = registry.system_prompt();
        assert!(prompt.contains("think"));
        assert!(prompt.contains("Action:"));
        assert!(prompt.contains("Answer:"));
    }

    #[test]
    fn test_fn_tool_wrapper() {
        let tool = FnTool::new(
            "ping",
            "Ping the system",
            serde_json::json!({}),
            Arc::new(|_| ToolResult::ok("ping", "pong")),
        );
        assert_eq!(tool.name(), "ping");
    }
}
