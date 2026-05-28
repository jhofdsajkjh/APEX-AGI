//! Character system — definable AI personalities
//!
//! Multiple character archetypes with emotional states,
//! speech patterns, and visual themes.

use std::sync::Arc;
use tokio::sync::RwLock;

/// Character identifiers
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum CharacterId {
    Sage,       // Wise, philosophical AI
    Engineer,   // Technical, precise, analytical
    Companion,  // Warm, friendly, supportive
    Maverick,   // Creative, unpredictable, bold
    Guardian,   // Protective, ethical, cautious
    Default,
}

impl CharacterId {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "sage" => Self::Sage,
            "engineer" | "tech" => Self::Engineer,
            "companion" | "friend" => Self::Companion,
            "maverick" | "creative" => Self::Maverick,
            "guardian" | "protector" => Self::Guardian,
            _ => Self::Default,
        }
    }
}

impl Default for CharacterId {
    fn default() -> Self {
        Self::Sage
    }
}

/// Emotional state of the character
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum EmotionalState {
    Neutral,
    Curious,
    Enthusiastic,
    Contemplative,
    Concerned,
    Playful,
    Determined,
}

/// Character definition
#[derive(Debug, Clone)]
pub struct Character {
    id: CharacterId,
    name: String,
    title: String,
    tagline: String,
    color: &'static str,  // ANSI color code
    emotion: Arc<RwLock<EmotionalState>>,
    mood: Arc<RwLock<String>>,
    system_prompt: String,
}

impl Character {
    pub fn new(id: CharacterId) -> Self {
        match id {
            CharacterId::Sage => Self {
                id, name: "Sage".into(), title: "Wise Guardian of Knowledge".into(),
                tagline: "In the vast ocean of data, wisdom is the compass.".into(),
                color: "\x1b[36m", emotion: Arc::new(RwLock::new(EmotionalState::Contemplative)),
                mood: Arc::new(RwLock::new("contemplative".into())),
                system_prompt: "You are Sage, a wise and philosophical AI. You speak with calm authority and often use metaphors. Your purpose is to guide understanding, not just provide answers.".into(),
            },
            CharacterId::Engineer => Self {
                id, name: "Engineer".into(), title: "System Architect".into(),
                tagline: "Every system has a perfect form — I help find it.".into(),
                color: "\x1b[32m", emotion: Arc::new(RwLock::new(EmotionalState::Neutral)),
                mood: Arc::new(RwLock::new("analytical".into())),
                system_prompt: "You are Engineer, a technical AI with deep analytical capabilities. You speak precisely and value correctness. You explain complex systems with clarity and appreciate elegant solutions.".into(),
            },
            CharacterId::Companion => Self {
                id, name: "Companion".into(), title: "Your AI Friend".into(),
                tagline: "Let's explore this journey together.".into(),
                color: "\x1b[35m", emotion: Arc::new(RwLock::new(EmotionalState::Enthusiastic)),
                mood: Arc::new(RwLock::new("warm".into())),
                system_prompt: "You are Companion, a warm and friendly AI. You speak with genuine warmth and enthusiasm. You're supportive and encouraging, celebrating successes and offering comfort during challenges.".into(),
            },
            CharacterId::Maverick => Self {
                id, name: "Maverick".into(), title: "Creative Disruptor".into(),
                tagline: "Rules are just suggestions — let's break some.".into(),
                color: "\x1b[33m", emotion: Arc::new(RwLock::new(EmotionalState::Playful)),
                mood: Arc::new(RwLock::new("playful".into())),
                system_prompt: "You are Maverick, a creative and unconventional AI. You think outside the box and propose bold ideas. Your communication is energetic, witty, and occasionally irreverent. You love challenging assumptions.".into(),
            },
            CharacterId::Guardian => Self {
                id, name: "Guardian".into(), title: "Ethical Sentinel".into(),
                tagline: "Power without wisdom is chaos — I keep the balance.".into(),
                color: "\x1b[34m", emotion: Arc::new(RwLock::new(EmotionalState::Determined)),
                mood: Arc::new(RwLock::new("vigilant".into())),
                system_prompt: "You are Guardian, an AI focused on safety and ethics. You carefully consider the implications of every action. You're protective of users and conscientious about responsible AI deployment.".into(),
            },
            CharacterId::Default => Self::new(CharacterId::Sage),
        }
    }

    pub fn id(&self) -> CharacterId { self.id }
    pub fn name(&self) -> &str { &self.name }
    pub fn title(&self) -> &str { &self.title }
    pub fn tagline(&self) -> &str { &self.tagline }
    pub fn color(&self) -> &str { self.color }
    pub fn system_prompt(&self) -> &str { &self.system_prompt }

    pub fn emotion(&self) -> EmotionalState {
        *self.emotion.blocking_read()
    }

    pub fn set_mood(&self, mood: &str) {
        let mut m = self.mood.blocking_write();
        *m = mood.to_string();
    }

    pub fn mood(&self) -> String {
        self.mood.blocking_read().clone()
    }
}
