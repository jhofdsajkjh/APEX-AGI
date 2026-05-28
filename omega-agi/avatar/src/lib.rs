//! # OMEGA AGI - Avatar Engine (Layer 9)
//!
//! Local human-like AI avatar with terminal interface:
//! - **Character**: definable personalities with emotional states
//! - **Chat**: natural conversation with LLM integration
//! - **TUI**: beautiful terminal user interface
//! - **Display**: colored message rendering with avatars
//!
//! ## Architecture
//!
//! ```text
//! AvatarEngine
//! ├── Character  (personality + emotion)
//! ├── Chat       (conversation engine)
//! ├── TUI        (terminal interface)
//! └── Display    (rendering + formatting)
//! ```

pub mod character;
pub mod chat;
pub mod tui;
pub mod display;

use std::sync::Arc;
use tokio::sync::RwLock;
use character::{Character, CharacterId, EmotionalState};
use chat::{ChatEngine, ChatMessage, ChatHistory};
use display::DisplayRenderer;
use tui::TuiSession;

/// Avatar configuration
#[derive(Debug, Clone)]
pub struct AvatarConfig {
    /// Active character ID
    pub character_id: CharacterId,
    /// TUI refresh rate (ms)
    pub tui_refresh_ms: u64,
    /// Show thinking/reasoning
    pub show_thinking: bool,
    /// Max chat history length
    pub max_history: usize,
    /// Enable typing animation
    pub typing_animation: bool,
    /// System prompt prefix
    pub system_prefix: String,
}

impl Default for AvatarConfig {
    fn default() -> Self {
        Self {
            character_id: CharacterId::default(),
            tui_refresh_ms: 100,
            show_thinking: true,
            max_history: 100,
            typing_animation: true,
            system_prefix: String::new(),
        }
    }
}

/// Avatar session summary
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AvatarSession {
    pub character: String,
    pub message_count: usize,
    pub duration_secs: u64,
    pub emotional_trajectory: Vec<String>,
    pub last_message: Option<String>,
}

/// The main Avatar engine — local human-like AI interaction
pub struct AvatarEngine {
    config: Arc<RwLock<AvatarConfig>>,
    character: Arc<RwLock<Character>>,
    chat: Arc<RwLock<ChatEngine>>,
    display: DisplayRenderer,
    session_start: std::time::Instant,
}

impl AvatarEngine {
    pub fn new() -> Self {
        Self::with_config(AvatarConfig::default())
    }

    pub fn with_config(config: AvatarConfig) -> Self {
        let character = Character::new(config.character_id);
        let chat = ChatEngine::new(
            character.name(),
            config.max_history,
            &config.system_prefix,
        );
        Self {
            config: Arc::new(RwLock::new(config)),
            character: Arc::new(RwLock::new(character)),
            chat: Arc::new(RwLock::new(chat)),
            display: DisplayRenderer::new(),
            session_start: std::time::Instant::now(),
        }
    }

    /// Send a message to the avatar and get a reply
    pub async fn chat(&self, user_input: &str) -> anyhow::Result<String> {
        let reply = {
            let chat = self.chat.read().await;
            chat.respond(user_input)
        };
        Ok(reply)
    }

    /// Add a message to the conversation history
    pub async fn add_message(&self, role: &str, content: &str) {
        let mut chat = self.chat.write().await;
        chat.add_message(role, content);
        let char_name = {
            let character = self.character.read().await;
            character.name().to_string()
        };
        self.display.show_message(role, &char_name, content);
    }

    /// Switch avatar character
    pub async fn switch_character(&self, id: CharacterId) {
        let mut character = self.character.write().await;
        *character = Character::new(id);
        let mut chat = self.chat.write().await;
        chat.reset(character.name());
        tracing::info!(character = %character.name(), "Switched avatar character");
    }

    /// Get current character info
    pub async fn character_info(&self) -> String {
        let character = self.character.read().await;
        format!(
            "{} — {}\n>>> {}",
            character.name(),
            character.title(),
            character.tagline()
        )
    }

    /// Get emotional state
    pub async fn emotion(&self) -> EmotionalState {
        self.character.read().await.emotion()
    }

    /// Set avatar mood
    pub async fn set_mood(&self, mood: &str) {
        let mut character = self.character.write().await;
        character.set_mood(mood);
    }

    /// Get conversation history
    pub async fn get_history(&self) -> Vec<ChatMessage> {
        let chat = self.chat.read().await;
        chat.history().clone()
    }

    /// Launch TUI interface
    pub async fn launch_tui(&self) -> anyhow::Result<()> {
        let session = TuiSession::new(
            self.character.clone(),
            self.chat.clone(),
            self.config.clone(),
        );
        session.run().await
    }

    /// Get session summary
    pub async fn session_info(&self) -> AvatarSession {
        let duration = self.session_start.elapsed().as_secs();
        let character = self.character.read().await;
        let chat = self.chat.read().await;
        let history = chat.history();
        let emotions: Vec<String> = history
            .iter()
            .filter_map(|m| {
                if m.role == "assistant" {
                    Some(format!("{:.20}", m.content))
                } else {
                    None
                }
            })
            .collect();

        AvatarSession {
            character: character.name().to_string(),
            message_count: history.len(),
            duration_secs: duration,
            emotional_trajectory: emotions.iter().rev().take(5).cloned().collect(),
            last_message: history.last().map(|m| m.content[..100.min(m.content.len())].to_string()),
        }
    }

    /// Version string
    pub fn version() -> &'static str {
        env!("CARGO_PKG_VERSION")
    }
}

impl Default for AvatarEngine {
    fn default() -> Self {
        Self::new()
    }
}
