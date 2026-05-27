//! # HyperCore Errors
//! Common error types for the HyperCore layer.

use thiserror::Error;

/// Unified error type for HyperCore operations.
#[derive(Error, Debug, Clone)]
pub enum HyperCoreError {
    #[error("Core error: {0}")]
    CoreError(String),

    #[error("Scheduler error: {0}")]
    SchedulerError(String),

    #[error("Memory error: {0}")]
    MemoryError(String),

    #[error("Security error: {0}")]
    SecurityError(String),

    #[error("Session error: {0}")]
    SessionError(String),

    #[error("Health check failed: {0}")]
    HealthCheckFailed(String),

    #[error("Pipeline error: {0}")]
    PipelineError(String),

    #[error("Diagnostics error: {0}")]
    DiagnosticsError(String),

    #[error("Self-heal error: {0}")]
    SelfHealError(String),

    #[error("IO error: {0}")]
    IoError(String),
}

impl From<std::io::Error> for HyperCoreError {
    fn from(e: std::io::Error) -> Self {
        HyperCoreError::IoError(e.to_string())
    }
}

impl From<String> for HyperCoreError {
    fn from(e: String) -> Self {
        HyperCoreError::CoreError(e)
    }
}
