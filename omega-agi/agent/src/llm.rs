//! LLM API client — OpenAI-compatible chat completions.
//! Supports any provider that exposes `/v1/chat/completions` (OpenAI, Claude via proxy,
//! local Ollama, vLLM, etc.).

use serde::{Deserialize, Serialize};

/// Simple inline macro to trim leading whitespace from multi-line strings.
macro_rules! dedent {
    ($s:expr) => {{
        let s = $s;
        let lines: Vec<&str> = s.lines().collect();
        let trimmed: Vec<&str> = lines.iter().map(|l| l.trim_start()).collect();
        // Drop first line if empty
        let start = trimmed.first().map(|l| l.is_empty()).unwrap_or(false) as usize;
        trimmed[start..].join("\n")
    }};
}

// ---------------------------------------------------------------------------
// Config
// ---------------------------------------------------------------------------

/// LLM provider configuration. All fields have sensible defaults for OpenAI.
#[derive(Debug, Clone)]
pub struct LLMConfig {
    pub api_url: String,
    pub api_key: String,
    pub model: String,
    pub max_tokens: u32,
    pub temperature: f32,
}

impl Default for LLMConfig {
    fn default() -> Self {
        Self {
            api_url: "https://api.openai.com/v1".into(),
            api_key: String::new(),
            model: "gpt-4o".into(),
            max_tokens: 4096,
            temperature: 0.7,
        }
    }
}

// ---------------------------------------------------------------------------
// Message types
// ---------------------------------------------------------------------------

/// A single chat message (role + content).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: String,
}

impl Message {
    pub fn system(content: impl Into<String>) -> Self {
        Self { role: "system".into(), content: content.into() }
    }
    pub fn user(content: impl Into<String>) -> Self {
        Self { role: "user".into(), content: content.into() }
    }
    pub fn assistant(content: impl Into<String>) -> Self {
        Self { role: "assistant".into(), content: content.into() }
    }
}

// ---------------------------------------------------------------------------
// Request / response wire types
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<Message>,
    max_tokens: u32,
    temperature: f32,
}

#[derive(Debug, Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
    usage: Option<Usage>,
}

#[derive(Debug, Deserialize)]
struct Choice {
    message: Message,
    finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Usage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

// ---------------------------------------------------------------------------
// Client
// ---------------------------------------------------------------------------

/// HTTP client for LLM chat completions.
pub struct LLMClient {
    config: LLMConfig,
    client: reqwest::Client,
}

impl LLMClient {
    /// Create a new client with the given configuration.
    pub fn new(config: LLMConfig) -> Self {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(180))
            .build()
            .expect("Failed to build HTTP client");
        Self { config, client }
    }

    /// Build config from environment variables (with the same names as `.env.example`).
    pub fn from_env() -> Self {
        let api_url = std::env::var("OMEGA_API_URL")
            .unwrap_or_else(|_| "https://api.openai.com/v1".into());
        let api_key = std::env::var("OMEGA_API_KEY").unwrap_or_default();
        let model = std::env::var("OMEGA_MODEL_NAME")
            .unwrap_or_else(|_| "gpt-4o".into());
        let max_tokens = std::env::var("OMEGA_MAX_TOKENS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(4096);
        let temperature = std::env::var("OMEGA_TEMPERATURE")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(0.7);

        Self::new(LLMConfig { api_url, api_key, model, max_tokens, temperature })
    }

    /// Send a chat completion request and return the assistant's reply text.
    pub async fn chat(&self, messages: &[Message]) -> anyhow::Result<String> {
        // If no API key is configured, return a mock so the system is still usable
        // for development / demonstration.
        if self.config.api_key.is_empty() {
            return Ok(self.mock_response(messages));
        }

        let request = ChatRequest {
            model: self.config.model.clone(),
            messages: messages.to_vec(),
            max_tokens: self.config.max_tokens,
            temperature: self.config.temperature,
        };

        let url = format!("{}/chat/completions", self.config.api_url.trim_end_matches('/'));

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("LLM API error ({}): {}", status, body);
        }

        let chat: ChatResponse = response.json().await?;
        chat.choices
            .into_iter()
            .next()
            .map(|c| c.message.content)
            .ok_or_else(|| anyhow::anyhow!("LLM returned empty choices"))
    }

    // ── mock fallback (used when no API key is set) ──────────────────────
    fn mock_response(&self, messages: &[Message]) -> String {
        // Extract the last user message for context
        let user_msg = messages
            .iter()
            .rev()
            .find(|m| m.role == "user")
            .map(|m| m.content.as_str())
            .unwrap_or("");

        // Determine the task from the user message
        if user_msg.contains("健康") || user_msg.contains("health") || user_msg.contains("检查") {
            return dedent!(r#"
                Thought: The user wants a health check. I'll use the health tool.
                Action: health
                Action Input: {}
            "#).to_string();
        }

        if user_msg.contains("诊断") || user_msg.contains("diagnos") {
            return dedent!(r#"
                Thought: The user wants a full system diagnostic. I'll run diagnostics.
                Action: diagnose
                Action Input: {}
            "#).to_string();
        }

        if user_msg.contains("代码") || user_msg.contains("code") || user_msg.contains("生成") {
            return dedent!(r#"
                Thought: The user wants code generation. I'll ask for details first.
                Answer: 我可以生成代码！请告诉我你想要什么语言的代码（Rust / Python），以及具体功能描述。
            "#).to_string();
        }

        // Generic fallback
        dedent!(r#"
            Thought: I'll first check the system health to understand the current state.
            Action: health
            Action Input: {}
        "#)
        .to_string()
    }
}


