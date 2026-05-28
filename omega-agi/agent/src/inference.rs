//! # InferenceEngine Trait — Multi-backend LLM abstraction
//!
//! The single most important architectural layer: allows the Agent to use
//! any LLM backend (OpenAI, Candle, Anthropic, Ollama, vLLM, mock) through
//! a uniform interface. New backends = just implement this trait.
//!
//! ## Backends
//!
//! | Engine          | Use case              | Requires            |
//! |-----------------|-----------------------|---------------------|
//! | `OpenAIEngine`  | Production (best)     | `OMEGA_API_KEY`     |
//! | `MockEngine`    | Testing / demo        | Nothing             |
//! | `CandleEngine`  | Local offline (WIP)   | Model files on disk |
//! | `RouterEngine`  | Auto-route by task    | Sub-engines         |
//!
//! ## Example
//!
//! ```ignore
//! let engine: Box<dyn InferenceEngine> = Box::new(OpenAIEngine::from_env());
//! let reply = engine.chat(&[Message::user("Hello")]).await?;
//! ```

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Message types (shared across all engines)
// ---------------------------------------------------------------------------

/// A single chat message with role and content.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

impl Message {
    pub fn system(content: impl Into<String>) -> Self {
        Self { role: "system".into(), content: content.into(), name: None }
    }
    pub fn user(content: impl Into<String>) -> Self {
        Self { role: "user".into(), content: content.into(), name: None }
    }
    pub fn assistant(content: impl Into<String>) -> Self {
        Self { role: "assistant".into(), content: content.into(), name: None }
    }
    pub fn tool( name: impl Into<String>, content: impl Into<String>) -> Self {
        Self { role: "tool".into(), content: content.into(), name: Some(name.into()) }
    }
}

// ---------------------------------------------------------------------------
// Chat response (unified)
// ---------------------------------------------------------------------------

/// Unified chat completion result.
#[derive(Debug, Clone)]
pub struct ChatResponse {
    pub content: String,
    pub model: String,
    pub usage: Option<TokenUsage>,
    pub latency_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

// ---------------------------------------------------------------------------
// Embedding types
// ---------------------------------------------------------------------------

/// A single embedding vector with metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Embedding {
    pub vector: Vec<f32>,
    pub dimensions: usize,
}

/// Result of an embedding request.
#[derive(Debug, Clone)]
pub struct EmbeddingResult {
    pub embeddings: Vec<Embedding>,
    pub model: String,
    pub latency_ms: u64,
}

// ---------------------------------------------------------------------------
// InferenceEngine trait
// ---------------------------------------------------------------------------

/// Abstract inference engine — all LLM backends implement this.
///
/// This is the **core abstraction** that makes the Agent provider-agnostic.
/// Add a new provider by implementing this trait on a new struct.
#[async_trait]
pub trait InferenceEngine: Send + Sync {
    /// Name of this engine (e.g. "openai", "mock", "candle").
    fn name(&self) -> &str;

    /// Chat completion — the primary interface.
    async fn chat(&self, messages: &[Message]) -> anyhow::Result<ChatResponse>;

    /// Streaming chat completion (optional — default falls back to non-streaming).
    async fn chat_stream(
        &self,
        messages: &[Message],
    ) -> anyhow::Result<Box<dyn tokio::io::AsyncBufRead + Send + Unpin>> {
        // Default: no streaming support, use non-streaming
        let _ = messages;
        anyhow::bail!("Streaming not supported by engine '{}'", self.name());
    }

    /// Generate embeddings for text chunks.
    /// Returns one embedding per chunk.
    async fn embed(&self, texts: &[&str]) -> anyhow::Result<EmbeddingResult> {
        anyhow::bail!("Embedding not supported by engine '{}'", self.name());
    }

    /// Count tokens in a text (approximate).
    fn count_tokens(&self, text: &str) -> usize {
        // Rough: ~4 chars per token for English, ~2 for CJK
        let cjk: usize = text.chars().filter(|&c| c as u32 > 0x2E80).count();
        let ascii = text.len().saturating_sub(cjk * 3);
        ascii / 4 + cjk / 2 + 1
    }

    /// Maximum context length for this model.
    fn max_context(&self) -> usize;

    /// Clone the engine into a box (trait-object-safe clone).
    fn box_clone(&self) -> Box<dyn InferenceEngine>;
}

// ---------------------------------------------------------------------------
// Helper: JSON schema for tool calling (OpenAI-compatible format)
// ---------------------------------------------------------------------------

/// Describes a tool for function-calling APIs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameters: Option<serde_json::Value>,
}
