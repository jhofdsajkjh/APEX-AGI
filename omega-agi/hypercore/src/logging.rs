//! # HyperCore Logging
//! Structured logging configuration and helpers.

use std::sync::atomic::{AtomicU64, Ordering};

/// Logging configuration.
#[derive(Debug, Clone)]
pub struct LoggingConfig {
    pub level: String,
    pub enable_file_logging: bool,
    pub log_dir: String,
    pub max_log_files: u32,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            enable_file_logging: false,
            log_dir: "./logs".to_string(),
            max_log_files: 10,
        }
    }
}

/// Simple structured logger wrapper.
pub struct Logger {
    config: LoggingConfig,
    event_counter: AtomicU64,
}

impl Logger {
    pub fn new(config: LoggingConfig) -> Self {
        Self {
            config,
            event_counter: AtomicU64::new(0),
        }
    }

    pub fn info(&self, msg: &str) {
        let id = self.event_counter.fetch_add(1, Ordering::SeqCst);
        tracing::info!(event_id = id, "{}", msg);
    }

    pub fn warn(&self, msg: &str) {
        let id = self.event_counter.fetch_add(1, Ordering::SeqCst);
        tracing::warn!(event_id = id, "{}", msg);
    }

    pub fn error(&self, msg: &str) {
        let id = self.event_counter.fetch_add(1, Ordering::SeqCst);
        tracing::error!(event_id = id, "{}", msg);
    }

    pub fn debug(&self, msg: &str) {
        let id = self.event_counter.fetch_add(1, Ordering::SeqCst);
        tracing::debug!(event_id = id, "{}", msg);
    }

    pub fn level(&self) -> &str {
        &self.config.level
    }
}

impl Default for Logger {
    fn default() -> Self {
        Self::new(LoggingConfig::default())
    }
}
