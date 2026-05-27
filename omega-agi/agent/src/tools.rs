//! Tool system — the bridge between the LLM Agent and the OMEGA AGI layers.
//!
//! The `ToolContext` trait is implemented by the main binary and provides
//! concrete access to all five layers (HyperCore, Runtime, Engineering,
//! Evolution, Adapters) plus filesystem operations.

use async_trait::async_trait;
use serde::Serialize;

// ---------------------------------------------------------------------------
// Result type
// ---------------------------------------------------------------------------

/// The outcome of a single tool invocation.
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

// ---------------------------------------------------------------------------
// Tool descriptor (used in the system prompt)
// ---------------------------------------------------------------------------

/// Describes a tool to the LLM — name, description, and JSON-schema-like args.
#[derive(Debug, Clone, Serialize)]
pub struct ToolDescriptor {
    pub name: &'static str,
    pub description: &'static str,
    pub args_json_schema: &'static str,
}

/// All tools the agent can call. Each entry becomes a bullet in the system prompt.
pub static ALL_TOOLS: &[ToolDescriptor] = &[
    ToolDescriptor {
        name: "health",
        description: "Check the overall health of the OMEGA AGI system. Returns status of all subsystems (hypercore, runtime, engineering, evolution, adapters).",
        args_json_schema: r#"{}"#,
    },
    ToolDescriptor {
        name: "diagnose",
        description: "Run a full diagnostic of the system. Returns detailed health report with subsystem-level information.",
        args_json_schema: r#"{}"#,
    },
    ToolDescriptor {
        name: "heal",
        description: "Attempt to automatically heal a subsystem that is in an unhealthy state.",
        args_json_schema: r#"{"type":"object","properties":{"target":{"type":"string","description":"Subsystem name to heal"},"issue":{"type":"string","description":"Description of the issue"}},"required":["target","issue"]}"#,
    },
    ToolDescriptor {
        name: "codegen",
        description: "Generate code in a specified language based on a natural-language description.",
        args_json_schema: r#"{"type":"object","properties":{"description":{"type":"string","description":"What the code should do"},"language":{"type":"string","enum":["rust","python"],"description":"Target programming language"}},"required":["description","language"]}"#,
    },
    ToolDescriptor {
        name: "evolve",
        description: "Run one cycle of the evolution engine. Mutates the current genome and evaluates the new configuration.",
        args_json_schema: r#"{}"#,
    },
    ToolDescriptor {
        name: "read",
        description: "Read the contents of a file from disk.",
        args_json_schema: r#"{"type":"object","properties":{"path":{"type":"string","description":"Absolute path to the file"}},"required":["path"]}"#,
    },
    ToolDescriptor {
        name: "write",
        description: "Write content to a file on disk. Creates directories as needed.",
        args_json_schema: r#"{"type":"object","properties":{"path":{"type":"string","description":"Absolute path to the file"},"content":{"type":"string","description":"Content to write"}},"required":["path","content"]}"#,
    },
    ToolDescriptor {
        name: "search",
        description: "Search for a pattern in the project source code (grep).",
        args_json_schema: r#"{"type":"object","properties":{"pattern":{"type":"string","description":"Text or regex pattern to search for"}},"required":["pattern"]}"#,
    },
    ToolDescriptor {
        name: "ls",
        description: "List files and directories at a given path.",
        args_json_schema: r#"{"type":"object","properties":{"path":{"type":"string","description":"Directory path to list"}},"required":["path"]}"#,
    },
    ToolDescriptor {
        name: "bash",
        description: "Execute a shell command and return its output.",
        args_json_schema: r#"{"type":"object","properties":{"command":{"type":"string","description":"Shell command to execute"}},"required":["command"]}"#,
    },
];

// ---------------------------------------------------------------------------
// Tool context trait
// ---------------------------------------------------------------------------

/// Abstract interface that the host application implements to give the agent
/// access to the five OMEGA AGI layers plus the filesystem.
#[async_trait]
pub trait ToolContext: Send + Sync {
    /// Execute a named tool with the given JSON arguments.
    async fn execute_tool(&self, name: &str, args: &serde_json::Value) -> ToolResult;
}

/// Render the tool descriptions into a system-prompt-friendly string.
pub fn tool_descriptions_prompt() -> String {
    let mut out = String::from("You have access to the following tools:\n\n");
    for t in ALL_TOOLS {
        out.push_str(&format!("- **{}**: {}  \n  Args JSON schema: `{}`\n\n", t.name, t.description, t.args_json_schema));
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
