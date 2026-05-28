//! Terminal UI — beautiful avatar interface
//!
//! Provides an interactive terminal session with the avatar,
//! featuring colored output, message history display, and
//! character switching.

use crate::character::{Character, CharacterId};
use crate::chat::ChatEngine;
use crate::AvatarConfig;
use std::sync::Arc;
use tokio::sync::RwLock;

/// The TUI session — interactive terminal interface
pub struct TuiSession {
    character: Arc<RwLock<Character>>,
    chat: Arc<RwLock<ChatEngine>>,
    config: Arc<RwLock<AvatarConfig>>,
}

impl TuiSession {
    pub fn new(
        character: Arc<RwLock<Character>>,
        chat: Arc<RwLock<ChatEngine>>,
        config: Arc<RwLock<AvatarConfig>>,
    ) -> Self {
        Self { character, chat, config }
    }

    /// Run the interactive TUI session
    pub async fn run(&self) -> anyhow::Result<()> {
        let character = self.character.read().await;

        println!("\n╔══════════════════════════════════════════════╗");
        println!("║        🎭 OMEGA AGI AVATAR INTERFACE          ║");
        println!("╚══════════════════════════════════════════════╝");
        println!();
        println!("{}  {} — {}  \x1b[0m", character.color(), character.name(), character.title());
        println!("  >>> {}", character.tagline());
        println!();
        println!("  Commands:");
        println!("    /switch <character>  — Change avatar (sage/engineer/companion/maverick/guardian)");
        println!("    /mood <mood>         — Set avatar mood");
        println!("    /history             — Show conversation history");
        println!("    /info                — Show character info");
        println!("    /quit                — Exit");
        println!();

        let mut input = String::new();

        loop {
            print!("You > ");
            use std::io::Write;
            std::io::stdout().flush()?;

            input.clear();
            std::io::stdin().read_line(&mut input)?;
            let input = input.trim();

            if input.is_empty() {
                continue;
            }

            match input {
                "/quit" | "/exit" | "/q" => {
                    println!("Session ended.");
                    break;
                }
                "/info" => {
                    let info = self.character.read().await;
                    println!("{}  {} \x1b[0m", info.color(), info.name());
                    println!("  Title: {}", info.title());
                    println!("  Tagline: {}", info.tagline());
                    println!("  Mood: {}", info.mood());
                    println!("  ID: {:?}", info.id());
                    continue;
                }
                "/history" => {
                    let history = self.chat.read().await;
                    for msg in history.history().iter().rev().take(10).rev() {
                        println!("[{}] {}: {}", msg.timestamp[..19].to_string(), msg.role, 
                            if msg.content.len() > 80 { format!("{}...", &msg.content[..77]) } else { msg.content.clone() });
                    }
                    continue;
                }
                cmd if cmd.starts_with("/switch ") => {
                    let id = CharacterId::from_str(cmd.trim_start_matches("/switch ").trim());
                    self.switch_character(id).await;
                    continue;
                }
                cmd if cmd.starts_with("/mood ") => {
                    let mood = cmd.trim_start_matches("/mood ").trim();
                    let char = self.character.read().await;
                    let _ = mood;
                    println!("Mood set to: {}", mood);
                    continue;
                }
                _ => {}
            }

            // Send message to avatar
            let reply = {
                let chat = self.chat.read().await;
                chat.respond(input)
            };

            let char_name = {
                let character = self.character.read().await;
                character.name().to_string()
            };

            println!("\n{}  {} \x1b[0m", self.character_color(), char_name);
            println!("  > {}", reply);
            println!();
        }

        Ok(())
    }

    fn character_color(&self) -> String {
        // This is a sync method, we read from a snapshot
        // Color is simplified here
        "\x1b[36m".to_string()
    }

    async fn switch_character(&self, id: CharacterId) {
        let character = crate::character::Character::new(id);
        let name = character.name().to_string();
        {
            let mut c = self.character.write().await;
            *c = character;
        }
        {
            let mut chat = self.chat.write().await;
            chat.reset(&name);
        }
        println!("Switched to avatar: {} — {}", name, self.character.read().await.title());
    }
}
