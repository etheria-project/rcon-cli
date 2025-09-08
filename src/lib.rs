//! RCON CLI Library
//!
//! A robust library and CLI tool for communicating with Minecraft servers
//! using the RCON (Remote Console) protocol.
//!
//! ## Features
//!
//! - Full RCON protocol implementation
//! - Async/await support with Tokio
//! - Command-line interface with CLAP
//! - Interactive and single-command modes
//! - Proper error handling and logging
//! - Connection pooling and retries
//! - Response fragmentation handling
//!
//! ## Example
//!
//! ```rust,no_run
//! use rcon_cli::{RconClient, RconConfig};
//! use std::net::SocketAddr;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let addr = "localhost:25575".parse::<SocketAddr>()?;
//!     let config = RconConfig::new(addr, "my_password");
//!
//!     let mut client = RconClient::connect(config).await?;
//!     let response = client.execute_command("list").await?;
//!
//!     println!("Server response: {}", response);
//!     Ok(())
//! }
//! ```

pub mod cli;
pub mod client;
pub mod error;
pub mod protocol;

// Re-export commonly used types
pub use cli::{Cli, Commands, OutputFormat, OutputFormatter};
pub use client::{RconClient, RconClientBuilder, RconConfig};
pub use error::{RconError, Result};
pub use protocol::{packet_type, RconPacket};

/// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Default RCON port
pub const DEFAULT_PORT: u16 = 25575;

/// Initialize logging for the library
pub fn init_logging(level: &str) -> Result<()> {
    use tracing_subscriber::{fmt, prelude::*, EnvFilter};

    let filter = EnvFilter::try_new(level)
        .map_err(|e| RconError::InvalidConfig(format!("Invalid log level: {}", e)))?;

    tracing_subscriber::registry()
        .with(fmt::layer().with_target(false).with_thread_ids(false))
        .with(filter)
        .try_init()
        .map_err(|e| RconError::InvalidConfig(format!("Failed to initialize logging: {}", e)))?;

    Ok(())
}
