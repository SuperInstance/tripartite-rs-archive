//! Error types for tripartite-rs

/// Core error type for tripartite-rs
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Agent not ready
    #[error("Agent '{0}' is not ready")]
    AgentNotReady(String),

    /// Agent processing failed
    #[error("Agent processing failed: {0}")]
    AgentProcessingFailed(String),

    /// Consensus not reached
    #[error("Consensus not reached after {0} rounds")]
    ConsensusNotReached(u8),

    /// Consensus vetoed
    #[error("Consensus vetoed: {0}")]
    ConsensusVetoed(String),

    /// Invalid configuration
    #[error("Invalid configuration: {0}")]
    InvalidConfiguration(String),

    /// Serialization error
    #[error("Serialization error: {0}")]
    SerializationError(String),

    /// I/O error
    #[error("I/O error: {0}")]
    IoError(String),

    /// Other error
    #[error("Error: {0}")]
    Other(String),
}

/// Result type for tripartite-rs
pub type Result<T> = std::result::Result<T, Error>;

// Implement conversions from common error types
impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Error::SerializationError(err.to_string())
    }
}

impl From<chrono::ParseError> for Error {
    fn from(err: chrono::ParseError) -> Self {
        Error::Other(format!("Time parsing error: {}", err))
    }
}
