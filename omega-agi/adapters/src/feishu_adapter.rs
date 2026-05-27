//! OMEGA AGI - Feishu (Lark) Adapter
//! 
//! Adapter for integrating with Feishu/Lark messaging platform.
//! Supports bot messaging, group management, and event handling.

use anyhow::{Context, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Feishu API endpoints
const FEISHU_BASE_URL: &str = "https://open.feishu.cn/open-apis";
const FEISHU_API_VERSION: &str = "v6";

/// Feishu event types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FeishuEventType {
    #[serde(rename = "im.message.receive_v1")]
    MessageReceive,
    #[serde(rename = "im.message")]
    Message,
    #[serde(rename = "contact.user.created")]
    UserCreated,
    #[serde(rename = "contact.user.updated")]
    UserUpdated,
    #[serde(rename = "calendar.event.created")]
    CalendarCreated,
    Unknown(String),
}

impl From<&str> for FeishuEventType {
    fn from(s: &str) -> Self {
        match s {
            "im.message.receive_v1" => FeishuEventType::MessageReceive,
            "im.message" => FeishuEventType::Message,
            "contact.user.created" => FeishuEventType::UserCreated,
            "contact.user.updated" => FeishuEventType::UserUpdated,
            "calendar.event.created" => FeishuEventType::CalendarCreated,
            other => FeishuEventType::Unknown(other.to_string()),
        }
    }
}

/// Feishu message types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum MessageType {
    Text,
    Post,
    Image,
    File,
    Audio,
    Media,
    Sticker,
    Interactive,
    ShareChat,
    ShareUser,
}

impl Default for MessageType {
    fn default() -> Self {
        MessageType::Text
    }
}

/// Feishu sender information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeishuSender {
    pub sender_id: FeishuSenderId,
    pub sender_type: String,
    pub tenant_key: String,
}

/// Feishu sender ID
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeishuSenderId {
    pub open_id: String,
    pub union_id: Option<String>,
    pub user_id: Option<String>,
}

/// Feishu message content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeishuMessageContent {
    pub message_id: String,
    pub create_time: String,
    pub chat_id: String,
    pub chat_type: String,
    pub message_type: MessageType,
    pub content: String,
    pub sender: FeishuSender,
}

/// Feishu message event from webhook
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeishuEvent {
    pub schema: String,
    pub header: FeishuEventHeader,
    pub event: Option<FeishuEventBody>,
}

/// Feishu event header
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeishuEventHeader {
    pub event_id: String,
    pub event_type: String,
    pub create_time: String,
    pub token: String,
    pub app_id: String,
    pub tenant_key: String,
}

/// Feishu event body
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeishuEventBody {
    pub sender: Option<FeishuSender>,
    pub message: Option<FeishuMessageContent>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// Feishu API response wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeishuResponse<T> {
    pub code: i32,
    pub msg: String,
    pub data: Option<T>,
}

impl<T> FeishuResponse<T> {
    pub fn is_success(&self) -> bool {
        self.code == 0
    }
    
    pub fn into_result(self) -> Result<T> {
        if self.code == 0 {
            self.data.context("No data in successful response")
        } else {
            anyhow::bail!("Feishu API error: {} - {}", self.code, self.msg)
        }
    }
}

/// Tenant access token response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantAccessToken {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: i64,
}

/// Send message request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendMessageRequest {
    pub receive_id: String,
    pub msg_type: MessageType,
    pub content: String,
}

/// Send message response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendMessageResponse {
    pub message_id: String,
}

/// Feishu configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FeishuConfig {
    pub app_id: String,
    pub app_secret: String,
    pub bot_name: Option<String>,
    pub webhook_url: Option<String>,
    pub enable_group_notifications: bool,
    pub notification_chat_id: Option<String>,
}

/// Feishu adapter for OMEGA AGI
#[derive(Debug)]
pub struct FeishuAdapter {
    config: Arc<RwLock<FeishuConfig>>,
    client: reqwest::Client,
    token_cache: Arc<RwLock<Option<(String, i64)>>>,
}

impl FeishuAdapter {
    /// Create a new Feishu adapter
    pub fn new(config: FeishuConfig) -> Self {
        Self {
            config: Arc::new(RwLock::new(config)),
            client: reqwest::Client::new(),
            token_cache: Arc::new(RwLock::new(None)),
        }
    }

    /// Create from environment variables
    pub fn from_env() -> Result<Self> {
        let app_id = std::env::var("FEISHU_APP_ID")
            .context("FEISHU_APP_ID not set")?;
        let app_secret = std::env::var("FEISHU_APP_SECRET")
            .context("FEISHU_APP_SECRET not set")?;
        
        let config = FeishuConfig {
            app_id,
            app_secret,
            bot_name: std::env::var("FEISHU_BOT_NAME").ok(),
            webhook_url: std::env::var("FEISHU_WEBHOOK_URL").ok(),
            enable_group_notifications: std::env::var("FEISHU_ENABLE_GROUP")
                .unwrap_or_default()
                .to_lowercase() == "true",
            notification_chat_id: std::env::var("FEISHU_NOTIFICATION_CHAT_ID").ok(),
        };
        
        Ok(Self::new(config))
    }

    /// Get tenant access token
    pub async fn get_access_token(&self) -> Result<String> {
        // Check cache first
        {
            let cache = self.token_cache.read().await;
            if let Some((ref token, expiry)) = *cache {
                if chrono::Utc::now().timestamp() < expiry - 300 {
                    return Ok(token.clone());
                }
            }
        }

        let config = self.config.read().await;
        let url = format!("{}/auth/v3/tenant_access_token/internal", FEISHU_BASE_URL);
        
        let body = serde_json::json!({
            "app_id": config.app_id,
            "app_secret": config.app_secret
        });

        let response = self.client
            .post(&url)
            .json(&body)
            .send()
            .await?
            .json::<FeishuResponse<TenantAccessToken>>()
            .await?
            .into_result()?;

        let token = response.access_token;
        let expiry = chrono::Utc::now().timestamp() + response.expires_in;

        {
            let mut cache = self.token_cache.write().await;
            *cache = Some((token.clone(), expiry));
        }

        Ok(token)
    }

    /// Send a text message
    pub async fn send_text(&self, receive_id: &str, content: &str) -> Result<String> {
        let url = format!("{}/im/v1/messages?receive_id_type=open_id", FEISHU_BASE_URL);
        let token = self.get_access_token().await?;

        let body = SendMessageRequest {
            receive_id: receive_id.to_string(),
            msg_type: MessageType::Text,
            content: serde_json::json!({ "text": content }).to_string(),
        };

        let response = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", token))
            .json(&body)
            .send()
            .await?
            .json::<FeishuResponse<SendMessageResponse>>()
            .await?
            .into_result()?;

        Ok(response.message_id)
    }

    /// Send a rich text message (post)
    pub async fn send_post(&self, receive_id: &str, title: &str, content: &str) -> Result<String> {
        let url = format!("{}/im/v1/messages?receive_id_type=open_id", FEISHU_BASE_URL);
        let token = self.get_access_token().await?;

        let post_content = serde_json::json!([
            [{"tag": "text", "text": title}],
            [{"tag": "text", "text": content}]
        ]);

        let body = SendMessageRequest {
            receive_id: receive_id.to_string(),
            msg_type: MessageType::Post,
            content: serde_json::json!({
                "zh_cn": {
                    "title": title,
                    "content": post_content
                }
            }).to_string(),
        };

        let response = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", token))
            .json(&body)
            .send()
            .await?
            .json::<FeishuResponse<SendMessageResponse>>()
            .await?
            .into_result()?;

        Ok(response.message_id)
    }

    /// Send interactive card message
    pub async fn send_card(&self, receive_id: &str, card_json: &str) -> Result<String> {
        let url = format!("{}/im/v1/messages?receive_id_type=open_id", FEISHU_BASE_URL);
        let token = self.get_access_token().await?;

        let body = serde_json::json!({
            "receive_id": receive_id,
            "msg_type": "interactive",
            "content": card_json
        });

        let response = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", token))
            .json(&body)
            .send()
            .await?
            .json::<FeishuResponse<SendMessageResponse>>()
            .await?
            .into_result()?;

        Ok(response.message_id)
    }

    /// Reply to a message
    pub async fn reply(&self, message_id: &str, content: &str) -> Result<String> {
        let url = format!("{}/im/v1/messages/{}/reply", FEISHU_BASE_URL, message_id);
        let token = self.get_access_token().await?;

        let body = serde_json::json!({
            "msg_type": "text",
            "content": serde_json::json!({ "text": content })
        });

        let response = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", token))
            .json(&body)
            .send()
            .await?
            .json::<FeishuResponse<SendMessageResponse>>()
            .await?
            .into_result()?;

        Ok(response.message_id)
    }

    /// Parse incoming event
    pub fn parse_event(&self, body: &[u8]) -> Result<FeishuEvent> {
        serde_json::from_slice(body)
            .context("Failed to parse Feishu event")
    }

    /// Extract text content from message event
    pub fn extract_text(&self, event: &FeishuEvent) -> Option<String> {
        let message = event.event.as_ref()?.message.as_ref()?;
        
        if message.message_type == MessageType::Text {
            let content: serde_json::Value = serde_json::from_str(&message.content).ok()?;
            content.get("text")?.as_str().map(String::from)
        } else {
            None
        }
    }

    /// Create a simple notification card
    pub fn create_notification_card(&self, title: &str, message: &str, level: &str) -> String {
        let color = match level {
            "error" | "critical" => "#FF3B30",
            "warning" | "warn" => "#FF9500",
            _ => "#34C759",
        };

        serde_json::json!({
            "config": { "wide_screen_mode": true },
            "elements": [
                {
                    "tag": "div",
                    "text": {
                        "tag": "lark_md",
                        "content": format!("**{}**", title)
                    }
                },
                {
                    "tag": "div",
                    "text": {
                        "tag": "lark_md",
                        "content": message
                    }
                },
                {
                    "tag": "hr"
                },
                {
                    "tag": "note",
                    "elements": [
                        {
                            "tag": "text",
                            "text": "OMEGA AGI Supremacy"
                        }
                    ]
                }
            ]
        }).to_string()
    }

    /// Health check - verify API connectivity
    pub async fn health_check(&self) -> Result<bool> {
        let token = self.get_access_token().await;
        Ok(token.is_ok())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_event_type() {
        assert!(matches!(
            FeishuEventType::from("im.message.receive_v1"),
            FeishuEventType::MessageReceive
        ));
        assert!(matches!(
            FeishuEventType::from("calendar.event.created"),
            FeishuEventType::CalendarCreated
        ));
    }

    #[test]
    fn test_message_type_default() {
        assert!(matches!(MessageType::default(), MessageType::Text));
    }

    #[test]
    fn test_response_is_success() {
        let success = FeishuResponse::<()>{ code: 0, msg: "success".to_string(), data: None };
        assert!(success.is_success());
        
        let failure = FeishuResponse::<()>{ code: 99991422, msg: "error".to_string(), data: None };
        assert!(!failure.is_success());
    }
}
