//! Tool system — updated to use Tool trait internally while keeping
//! the old ToolContext/ToolResult API for backward compatibility.

use async_trait::async_trait;
use serde::Serialize;
use crate::tool::{Tool, ToolRegistry, ToolResult as NewToolResult};
use std::sync::Arc;

// Re-export for backward compatibility
pub use crate::tool::ToolResult;

/// Tool descriptor (kept for backward compatibility).
#[derive(Debug, Clone, Serialize)]
pub struct ToolDescriptor {
    pub name: &'static str,
    pub description: &'static str,
    pub args_json_schema: &'static str,
}

/// All tools available to the Agent (kept for backward compatibility).
pub static ALL_TOOLS: &[ToolDescriptor] = &[
    ToolDescriptor {
        name: "health",
        description: "Check the overall health of the OMEGA AGI system.",
        args_json_schema: r#"{}"#,
    },
    ToolDescriptor {
        name: "diagnose",
        description: "Run a full diagnostic of the system.",
        args_json_schema: r#"{}"#,
    },
    ToolDescriptor {
        name: "heal",
        description: "Attempt to automatically heal a subsystem.",
        args_json_schema: r#"{"type":"object","properties":{"target":{"type":"string"},"issue":{"type":"string"}},"required":["target","issue"]}"#,
    },
    ToolDescriptor {
        name: "codegen",
        description: "Generate code in a specified language.",
        args_json_schema: r#"{"type":"object","properties":{"description":{"type":"string"},"language":{"type":"string","enum":["rust","python"]}},"required":["description","language"]}"#,
    },
    ToolDescriptor {
        name: "evolve",
        description: "Run one cycle of the evolution engine.",
        args_json_schema: r#"{}"#,
    },
    ToolDescriptor {
        name: "evolve_full",
        description: "Run the complete auto-evolve pipeline.",
        args_json_schema: r#"{}"#,
    },
    ToolDescriptor {
        name: "read",
        description: "Read a file from the filesystem.",
        args_json_schema: r#"{"type":"object","properties":{"path":{"type":"string"}},"required":["path"]}"#,
    },
    ToolDescriptor {
        name: "write",
        description: "Write content to a file.",
        args_json_schema: r#"{"type":"object","properties":{"path":{"type":"string"},"content":{"type":"string"}},"required":["path","content"]}"#,
    },
    ToolDescriptor {
        name: "search",
        description: "Search source code for a pattern.",
        args_json_schema: r#"{"type":"object","properties":{"pattern":{"type":"string"}},"required":["pattern"]}"#,
    },
    ToolDescriptor {
        name: "ls",
        description: "List directory contents.",
        args_json_schema: r#"{"type":"object","properties":{"path":{"type":"string"}},"required":["path"]}"#,
    },
    ToolDescriptor {
        name: "bash",
        description: "Execute a shell command.",
        args_json_schema: r#"{"type":"object","properties":{"command":{"type":"string"}},"required":["command"]}"#,
    },
];

/// Abstract interface for the host application (kept for backward compatibility).
#[async_trait]
pub trait ToolContext: Send + Sync {
    async fn execute_tool(&self, name: &str, args: &serde_json::Value) -> ToolResult;
}

/// Render tool descriptions for system prompt (kept for backward compatibility).
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
