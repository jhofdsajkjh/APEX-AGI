//! Chat engine — conversational AI interaction
//!
//! Manages conversation history, generates responses using
//! character-aware logic, and maintains context.

use crate::character::Character;
use std::sync::Arc;
use tokio::sync::RwLock;

/// A single chat message
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
    pub timestamp: String,
}

/// The ChatEngine — manages conversations with the avatar
pub struct ChatEngine {
    character_name: String,
    history: Arc<RwLock<Vec<ChatMessage>>>,
    max_history: usize,
    system_prompt: String,
}

impl ChatEngine {
    pub fn new(character_name: &str, max_history: usize, system_prefix: &str) -> Self {
        let system_prompt = if system_prefix.is_empty() {
            format!("You are {}, an AI avatar. Respond naturally and conversationally.", character_name)
        } else {
            format!("{}\nYou are {}.", system_prefix, character_name)
        };

        let mut engine = Self {
            character_name: character_name.to_string(),
            history: Arc::new(RwLock::new(Vec::new())),
            max_history,
            system_prompt,
        };

        // Add system message
        engine.add_message("system", &engine.system_prompt);

        engine
    }

    /// Generate a response to user input
    pub fn respond(&self, user_input: &str) -> String {
        self.add_message("user", user_input);

        // Generate a contextual response based on conversation history
        let response = self.generate_response(user_input);

        self.add_message("assistant", &response);
        response
    }

    fn generate_response(&self, _input: &str) -> String {
        // In production, this calls the LLM engine
        // For now, return character-aware template responses
        format!(
            "[{}] I'm processing your request. (LLM integration pending — set OMEGA_API_KEY for full AI responses)",
            self.character_name
        )
    }

    /// Add a message to history
    pub fn add_message(&self, role: &str, content: &str) {
        let mut history = self.history.blocking_write();
        history.push(ChatMessage {
            role: role.to_string(),
            content: content.to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
        });
        if history.len() > self.max_history {
            history.remove(1); // Keep system prompt at index 0
        }
    }

    /// Get conversation history
    pub fn history(&self) -> Vec<ChatMessage> {
        self.history.blocking_read().clone()
    }

    /// Reset conversation
    pub fn reset(&self, character_name: &str) {
        let mut history = self.history.blocking_write();
        history.clear();
        history.push(ChatMessage {
            role: "system".to_string(),
            content: format!("You are {}. Respond naturally.", character_name),
            timestamp: chrono::Utc::now().to_rfc3339(),
        });
    }

    /// Get context window (last N messages)
    pub fn context(&self, n: usize) -> Vec<ChatMessage> {
        let history = self.history.blocking_read();
        let start = if history.len() > n { history.len() - n } else { 0 };
        history[start..].to_vec()
    }
}
