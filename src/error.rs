use std::io;
use thiserror::Error;

/// Custom error types for the RCON CLI application
#[derive(Error, Debug)]
pub enum RconError {
    #[error("Network error: {0}")]
    Network(#[from] io::Error),

    #[error("Connection timeout")]
    Timeout,

    #[error("Authentication failed")]
    AuthenticationFailed,

    #[error("Invalid packet format: {0}")]
    InvalidPacket(String),

    #[error("Protocol error: {0}")]
    Protocol(String),

    #[error("Server disconnected")]
    Disconnected,

    #[error("Command execution failed: {0}")]
    CommandFailed(String),

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
}

/// Result type alias for convenience
pub type Result<T> = std::result::Result<T, RconError>;
