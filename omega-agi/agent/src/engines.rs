//! # Concrete Inference Engine Implementations
//!
//! Backends for the `InferenceEngine` trait:
//! - `OpenAIEngine` — production LLM via REST API
//! - `MockEngine` — deterministic responses for testing
//! - `RouterEngine` — auto-routes by task type

use async_trait::async_trait;
use std::time::Instant;

use super::inference::*;

// ---------------------------------------------------------------------------
// OpenAIEngine
// ---------------------------------------------------------------------------

/// Production engine — calls any OpenAI-compatible API.
pub struct OpenAIEngine {
    config: OpenAIEngineConfig,
    client: reqwest::Client,
}

#[derive(Debug, Clone)]
pub struct OpenAIEngineConfig {
    pub api_url: String,
    pub api_key: String,
    pub model: String,
    pub max_tokens: u32,
    pub temperature: f32,
    pub embed_model: String,
}

impl Default for OpenAIEngineConfig {
    fn default() -> Self {
        Self {
            api_url: std::env::var("OMEGA_API_URL")
                .unwrap_or_else(|_| "https://api.openai.com/v1".into()),
            api_key: std::env::var("OMEGA_API_KEY").unwrap_or_default(),
            model: std::env::var("OMEGA_MODEL_NAME").unwrap_or_else(|_| "gpt-4o".into()),
            max_tokens: std::env::var("OMEGA_MAX_TOKENS")
                .ok().and_then(|v| v.parse().ok())
                .unwrap_or(4096),
            temperature: std::env::var("OMEGA_TEMPERATURE")
                .ok().and_then(|v| v.parse().ok())
                .unwrap_or(0.7),
            embed_model: "text-embedding-3-small".into(),
        }
    }
}

impl OpenAIEngine {
    pub fn new(config: OpenAIEngineConfig) -> Self {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(180))
            .build()
            .expect("Failed to build HTTP client");
        Self { config, client }
    }

    /// Create from environment variables.
    pub fn from_env() -> Self {
        Self::new(OpenAIEngineConfig::default())
    }
}

#[async_trait]
impl InferenceEngine for OpenAIEngine {
    fn name(&self) -> &str { "openai" }

    async fn chat(&self, messages: &[Message]) -> anyhow::Result<ChatResponse> {
        let start = Instant::now();

        // Build request body
        let body = serde_json::json!({
            "model": self.config.model,
            "messages": messages,
            "max_tokens": self.config.max_tokens,
            "temperature": self.config.temperature,
        });

        let url = format!("{}/chat/completions", self.config.api_url.trim_end_matches('/'));
        let resp = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .json(&body)
            .send()
            .await?;

        let status = resp.status();
        if !status.is_success() {
            let body_text = resp.text().await.unwrap_or_default();
            anyhow::bail!("OpenAI API error ({}): {}", status, body_text);
        }

        let json: serde_json::Value = resp.json().await?;
        let latency = start.elapsed().as_millis() as u64;

        let content = json["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or("")
            .to_string();

        let usage = json["usage"].as_object().map(|u| TokenUsage {
            prompt_tokens: u.get("prompt_tokens").and_then(|v| v.as_u64()).unwrap_or(0) as u32,
            completion_tokens: u.get("completion_tokens").and_then(|v| v.as_u64()).unwrap_or(0) as u32,
            total_tokens: u.get("total_tokens").and_then(|v| v.as_u64()).unwrap_or(0) as u32,
        });

        Ok(ChatResponse {
            content,
            model: self.config.model.clone(),
            usage,
            latency_ms: latency,
        })
    }

    async fn embed(&self, texts: &[&str]) -> anyhow::Result<EmbeddingResult> {
        let start = Instant::now();

        let body = serde_json::json!({
            "model": self.config.embed_model,
            "input": texts,
        });

        let url = format!("{}/embeddings", self.config.api_url.trim_end_matches('/'));
        let resp = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .json(&body)
            .send()
            .await?;

        let status = resp.status();
        if !status.is_success() {
            let body_text = resp.text().await.unwrap_or_default();
            anyhow::bail!("OpenAI Embedding API error ({}): {}", status, body_text);
        }

        let json: serde_json::Value = resp.json().await?;
        let latency = start.elapsed().as_millis() as u64;

        let embeddings: Vec<Embedding> = json["data"]
            .as_array()
            .map(|arr| {
                arr.iter().map(|item| {
                    let vec: Vec<f32> = item["embedding"]
                        .as_array()
                        .map(|v| v.iter().map(|n| n.as_f64().unwrap_or(0.0) as f32).collect())
                        .unwrap_or_default();
                    Embedding {
                        dimensions: vec.len(),
                        vector: vec,
                    }
                }).collect()
            })
            .unwrap_or_default();

        Ok(EmbeddingResult {
            embeddings,
            model: self.config.embed_model.clone(),
            latency_ms: latency,
        })
    }

    fn max_context(&self) -> usize {
        if self.config.model.contains("gpt-4") { 128000 }
        else if self.config.model.contains("claude") { 200000 }
        else if self.config.model.contains("deepseek") { 65536 }
        else { 8192 }
    }

    fn box_clone(&self) -> Box<dyn InferenceEngine> {
        Box::new(Self {
            config: self.config.clone(),
            client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(180))
                .build()
                .expect("Failed to build HTTP client"),
        })
    }
}

// ---------------------------------------------------------------------------
// MockEngine (for testing / demo without API key)
// ---------------------------------------------------------------------------

/// Deterministic mock engine for testing — returns pre-configured responses.
pub struct MockEngine {
    /// Optional: override the default response pattern.
    pub responses: Vec<String>,
    response_index: std::sync::Mutex<usize>,
}

impl MockEngine {
    pub fn new() -> Self {
        Self {
            responses: vec![],
            response_index: std::sync::Mutex::new(0),
        }
    }

    pub fn with_responses(responses: Vec<String>) -> Self {
        Self {
            responses,
            response_index: std::sync::Mutex::new(0),
        }
    }
}

impl Default for MockEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl InferenceEngine for MockEngine {
    fn name(&self) -> &str { "mock" }

    async fn chat(&self, messages: &[Message]) -> anyhow::Result<ChatResponse> {
        // Simulate latency
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;

        let content = if !self.responses.is_empty() {
            let mut idx = self.response_index.lock().unwrap();
            let resp = self.responses[*idx % self.responses.len()].clone();
            *idx += 1;
            resp
        } else {
            // Default mock: respond based on last user message
            let user_msg = messages.iter().rev()
                .find(|m| m.role == "user")
                .map(|m| m.content.as_str())
                .unwrap_or("");

            let lower = user_msg.to_lowercase();
            if lower.contains("health") || lower.contains("健康") || lower.contains("检查") {
                "Thought: The user wants a health check. I'll use the health tool.\nAction: health\nAction Input: {}".to_string()
            } else if lower.contains("code") || lower.contains("代码") || lower.contains("生成") {
                "Thought: The user wants code generation.\nAnswer: I can generate code! What language would you like? (Rust/Python)".to_string()
            } else {
                format!("Thought: I received: \"{}\". Let me check system health first.\nAction: health\nAction Input: {{}}", user_msg)
            }
        };

        Ok(ChatResponse {
            content,
            model: "mock-model".into(),
            usage: Some(TokenUsage { prompt_tokens: 100, completion_tokens: 50, total_tokens: 150 }),
            latency_ms: 10,
        })
    }

    async fn embed(&self, texts: &[&str]) -> anyhow::Result<EmbeddingResult> {
        // Return random-ish deterministic embeddings for testing
        let embeddings: Vec<Embedding> = texts.iter().enumerate().map(|(i, _)| {
            let dim = 384;
            let seed = i as f32 * 0.1;
            let vector: Vec<f32> = (0..dim).map(|j| {
                ((seed + j as f32 * 0.01).sin() * 0.5 + 0.5) as f32
            }).collect();
            Embedding { dimensions: dim, vector }
        }).collect();

        Ok(EmbeddingResult {
            embeddings,
            model: "mock-embed".into(),
            latency_ms: 5,
        })
    }

    fn count_tokens(&self, text: &str) -> usize {
        text.len() / 4 + 1
    }

    fn max_context(&self) -> usize { 4096 }

    fn box_clone(&self) -> Box<dyn InferenceEngine> {
        Box::new(MockEngine {
            responses: self.responses.clone(),
            response_index: std::sync::Mutex::new(0),
        })
    }
}

// ---------------------------------------------------------------------------
// RouterEngine — routes requests to sub-engines based on task type
// ---------------------------------------------------------------------------

/// Routes chat requests to the best engine based on task analysis.
pub struct RouterEngine {
    pub default_engine: Box<dyn InferenceEngine>,
    pub code_engine: Option<Box<dyn InferenceEngine>>,
    pub fast_engine: Option<Box<dyn InferenceEngine>>,
    pub embed_engine: Option<Box<dyn InferenceEngine>>,
}

impl RouterEngine {
    pub fn new(default: Box<dyn InferenceEngine>) -> Self {
        Self {
            default_engine: default,
            code_engine: None,
            fast_engine: None,
            embed_engine: None,
        }
    }

    /// Determine which engine to use for a task.
    fn select_engine(&self, messages: &[Message]) -> &Box<dyn InferenceEngine> {
        let combined: String = messages.iter().map(|m| m.content.as_str()).collect();
        let lower = combined.to_lowercase();

        // Code generation → code engine (e.g., Claude 3.5 Sonnet or o1-mini)
        if (lower.contains("generate") || lower.contains("code") || lower.contains("implement"))
            && (lower.contains("function") || lower.contains("class") || lower.contains("rust"))
        {
            return self.code_engine.as_ref().unwrap_or(&self.default_engine);
        }

        // Fast / simple tasks → fast engine
        if lower.len() < 50 && (lower.contains("hi") || lower.contains("hello") || lower.contains("help")) {
            return self.fast_engine.as_ref().unwrap_or(&self.default_engine);
        }

        &self.default_engine
    }
}

#[async_trait]
impl InferenceEngine for RouterEngine {
    fn name(&self) -> &str { "router" }

    async fn chat(&self, messages: &[Message]) -> anyhow::Result<ChatResponse> {
        let engine = self.select_engine(messages);
        engine.chat(messages).await
    }

    async fn embed(&self, texts: &[&str]) -> anyhow::Result<EmbeddingResult> {
        match &self.embed_engine {
            Some(e) => e.embed(texts).await,
            None => self.default_engine.embed(texts).await,
        }
    }

    fn max_context(&self) -> usize { self.default_engine.max_context() }

    fn box_clone(&self) -> Box<dyn InferenceEngine> {
        Box::new(RouterEngine {
            default_engine: self.default_engine.box_clone(),
            code_engine: self.code_engine.as_ref().map(|e| e.box_clone()),
            fast_engine: self.fast_engine.as_ref().map(|e| e.box_clone()),
            embed_engine: self.embed_engine.as_ref().map(|e| e.box_clone()),
        })
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_engine_returns_response() {
        let engine = MockEngine::new();
        let resp = engine.chat(&[Message::user("hello")]).await.unwrap();
        assert!(!resp.content.is_empty());
        assert_eq!(resp.model, "mock-model");
    }

    #[tokio::test]
    async fn test_mock_engine_with_custom_responses() {
        let engine = MockEngine::with_responses(vec!["Answer: Hello!".into(), "Answer: World!".into()]);
        let r1 = engine.chat(&[]).await.unwrap();
        let r2 = engine.chat(&[]).await.unwrap();
        assert_eq!(r1.content, "Answer: Hello!");
        assert_eq!(r2.content, "Answer: World!");
    }

    #[tokio::test]
    async fn test_mock_embedding_dimensions() {
        let engine = MockEngine::new();
        let result = engine.embed(&["test text"]).await.unwrap();
        assert_eq!(result.embeddings.len(), 1);
        assert_eq!(result.embeddings[0].dimensions, 384);
    }

    #[test]
    fn test_mock_token_count() {
        let engine = MockEngine::new();
        assert!(engine.count_tokens("hello world") > 0);
    }

    #[test]
    fn test_router_engine_creation() {
        let default = Box::new(MockEngine::new());
        let router = RouterEngine::new(default);
        assert_eq!(router.name(), "router");
    }

    #[test]
    fn test_openai_config_default() {
        let config = OpenAIEngineConfig::default();
        assert_eq!(config.model, "gpt-4o");
        assert_eq!(config.max_tokens, 4096);
    }
}
