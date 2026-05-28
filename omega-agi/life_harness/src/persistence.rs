//! Session persistence — save and restore system state
//!
//! Maintains session state across restarts through file-based
//! persistence, enabling the system to resume where it left off.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// A stored session entry
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct SessionEntry {
    session_id: String,
    created: String,
    updated: String,
    data: HashMap<String, String>,
}

/// The SessionStore — persists sessions to disk
pub struct SessionStore {
    path: String,
    sessions: Arc<RwLock<HashMap<String, SessionEntry>>>,
}

impl SessionStore {
    pub fn new(path: &str) -> Self {
        Self {
            path: path.to_string(),
            sessions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Save current state to disk
    pub async fn save(&self) -> anyhow::Result<()> {
        let sessions = self.sessions.read().await;
        let json = serde_json::to_string_pretty(&*sessions)?;

        // Ensure directory exists
        if let Some(parent) = std::path::Path::new(&self.path).parent() {
            std::fs::create_dir_all(parent)?;
        }

        std::fs::write(&self.path, json)?;
        Ok(())
    }

    /// Restore state from disk
    pub async fn restore(&self) -> anyhow::Result<Vec<String>> {
        let content = std::fs::read_to_string(&self.path)?;
        let restored: HashMap<String, SessionEntry> = serde_json::from_str(&content)?;

        let keys: Vec<String> = restored.keys().cloned().collect();
        {
            let mut sessions = self.sessions.write().await;
            *sessions = restored;
        }

        Ok(keys)
    }

    /// Store a session key-value pair
    pub async fn store(&self, session_id: &str, key: &str, value: &str) {
        let now = chrono::Utc::now().to_rfc3339();
        let mut sessions = self.sessions.write().await;

        let entry = sessions.entry(session_id.to_string()).or_insert_with(|| SessionEntry {
            session_id: session_id.to_string(),
            created: now.clone(),
            updated: now.clone(),
            data: HashMap::new(),
        });

        entry.updated = now;
        entry.data.insert(key.to_string(), value.to_string());
    }

    /// Retrieve a session value
    pub async fn get(&self, session_id: &str, key: &str) -> Option<String> {
        let sessions = self.sessions.read().await;
        sessions.get(session_id)?.data.get(key).cloned()
    }

    /// Get session count
    pub async fn session_count(&self) -> usize {
        self.sessions.read().await.len()
    }

    /// Clear all sessions
    pub async fn clear(&self) {
        let mut sessions = self.sessions.write().await;
        sessions.clear();
        let _ = self.save().await;
    }
}
