use clap::{Parser, Subcommand, ValueEnum};
use std::net::SocketAddr;
use std::time::Duration;

/// CLI interface for the RCON client
#[derive(Parser)]
#[command(name = "rcon-cli")]
#[command(about = "A CLI client for Minecraft RCON protocol")]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(author = "RCON CLI Team")]
#[command(long_about = "
A robust command-line interface for communicating with Minecraft servers
using the RCON (Remote Console) protocol. Supports both single command
execution and interactive sessions.

Examples:
  rcon-cli -a localhost:25575 -p secret exec \"time set day\"
  rcon-cli -a localhost:25575 -p secret interactive
  rcon-cli -a play.example.com:25575 -p mypass ping
")]
pub struct Cli {
    /// Server address in format host:port
    #[arg(
        short = 'a',
        long = "address",
        default_value = "localhost:25575",
        help = "RCON server address (host:port)",
        value_name = "HOST:PORT"
    )]
    pub address: String,

    /// RCON password
    #[arg(short = 'p', long = "password", help = "RCON server password")]
    pub password: String,

    /// Connection timeout in seconds
    #[arg(
        short = 't',
        long = "timeout",
        default_value = "5",
        help = "Connection timeout in seconds",
        value_name = "SECONDS"
    )]
    pub timeout: u64,

    /// Logging level
    #[arg(
        short = 'v',
        long = "verbose",
        help = "Increase logging verbosity",
        action = clap::ArgAction::Count
    )]
    pub verbose: u8,

    /// Output format
    #[arg(
        short = 'f',
        long = "format",
        default_value = "text",
        help = "Output format"
    )]
    pub format: OutputFormat,

    /// Disable colored output
    #[arg(
        long = "no-color",
        help = "Disable colored output",
        action = clap::ArgAction::SetTrue
    )]
    pub no_color: bool,

    #[command(subcommand)]
    pub command: Commands,
}

/// Available output formats
#[derive(Debug, Clone, ValueEnum)]
pub enum OutputFormat {
    /// Plain text output (default)
    Text,
    /// JSON formatted output
    Json,
}

/// Available commands
#[derive(Subcommand)]
pub enum Commands {
    /// Execute a single command on the server
    #[command(alias = "run")]
    Exec {
        /// The command to execute on the server
        #[arg(
            help = "Command to execute (e.g., 'list', 'time set day')",
            value_name = "COMMAND"
        )]
        command: String,

        /// Show command execution time
        #[arg(
            long = "time",
            help = "Show command execution time",
            action = clap::ArgAction::SetTrue
        )]
        show_time: bool,
    },

    /// Start an interactive RCON session
    #[command(alias = "repl")]
    Interactive {
        /// Custom prompt for interactive mode
        #[arg(
            long = "prompt",
            default_value = "rcon> ",
            help = "Custom prompt for interactive mode"
        )]
        prompt: String,

        /// Enable command history
        #[arg(
            long = "history",
            help = "Enable command history (saves to ~/.rcon_history)",
            action = clap::ArgAction::SetTrue
        )]
        history: bool,

        /// Maximum number of history entries
        #[arg(
            long = "history-size",
            default_value = "1000",
            help = "Maximum number of history entries"
        )]
        history_size: usize,
    },

    /// Test connection to the RCON server
    Ping {
        /// Number of ping attempts
        #[arg(
            short = 'c',
            long = "count",
            default_value = "1",
            help = "Number of ping attempts"
        )]
        count: u32,

        /// Interval between pings in seconds
        #[arg(
            short = 'i',
            long = "interval",
            default_value = "1",
            help = "Interval between pings in seconds"
        )]
        interval: u64,
    },

    /// Show server information
    Info {
        /// Include detailed server stats
        #[arg(
            long = "detailed",
            help = "Include detailed server statistics",
            action = clap::ArgAction::SetTrue
        )]
        detailed: bool,
    },

    /// List online players
    Players {
        /// Show player UUIDs
        #[arg(
            long = "uuids",
            help = "Show player UUIDs",
            action = clap::ArgAction::SetTrue
        )]
        show_uuids: bool,
    },
}

impl Cli {
    /// Parse the address string and convert localhost to 127.0.0.1
    pub fn parse_address(&self) -> Result<SocketAddr, String> {
        let address_str = if self.address.starts_with("localhost:") {
            self.address.replace("localhost:", "127.0.0.1:")
        } else if self.address == "localhost" {
            "127.0.0.1".to_string()
        } else {
            self.address.clone()
        };

        address_str
            .parse::<SocketAddr>()
            .map_err(|e| format!("Invalid address format '{}': {}", self.address, e))
    }

    /// Get the connection timeout as a Duration
    pub fn timeout_duration(&self) -> Duration {
        Duration::from_secs(self.timeout)
    }

    /// Get the appropriate logging level based on verbosity
    pub fn log_level(&self) -> &'static str {
        match self.verbose {
            0 => "warn",
            1 => "info",
            2 => "debug",
            _ => "trace",
        }
    }

    /// Check if colors should be used for output
    pub fn use_colors(&self) -> bool {
        !self.no_color && atty::is(atty::Stream::Stdout)
    }

    /// Validate the CLI arguments
    pub fn validate(&self) -> Result<(), String> {
        // Validate timeout
        if self.timeout == 0 {
            return Err("Timeout must be greater than 0".to_string());
        }

        // Validate password is not empty
        if self.password.is_empty() {
            return Err("Password cannot be empty".to_string());
        }

        // Command-specific validation
        match &self.command {
            Commands::Exec { command, .. } => {
                if command.trim().is_empty() {
                    return Err("Command cannot be empty".to_string());
                }
            }
            Commands::Interactive { history_size, .. } => {
                if *history_size == 0 {
                    return Err("History size must be greater than 0".to_string());
                }
            }
            Commands::Ping {
                count, interval, ..
            } => {
                if *count == 0 {
                    return Err("Ping count must be greater than 0".to_string());
                }
                if *interval == 0 {
                    return Err("Ping interval must be greater than 0".to_string());
                }
            }
            _ => {}
        }

        Ok(())
    }
}

/// Helper struct for formatting command output
pub struct OutputFormatter {
    format: OutputFormat,
    use_colors: bool,
}

impl OutputFormatter {
    pub fn new(format: OutputFormat, use_colors: bool) -> Self {
        Self { format, use_colors }
    }

    pub fn format_response(&self, response: &str) -> String {
        match self.format {
            OutputFormat::Text => {
                if self.use_colors {
                    self.colorize_response(response)
                } else {
                    response.to_string()
                }
            }
            OutputFormat::Json => serde_json::json!({
                "response": response,
                "timestamp": chrono::Utc::now().to_rfc3339()
            })
            .to_string(),
        }
    }

    pub fn format_error(&self, error: &str) -> String {
        match self.format {
            OutputFormat::Text => {
                if self.use_colors {
                    format!("\x1b[31mError: {}\x1b[0m", error)
                } else {
                    format!("Error: {}", error)
                }
            }
            OutputFormat::Json => serde_json::json!({
                "error": error,
                "timestamp": chrono::Utc::now().to_rfc3339()
            })
            .to_string(),
        }
    }

    pub fn format_info(&self, info: &str) -> String {
        match self.format {
            OutputFormat::Text => {
                if self.use_colors {
                    format!("\x1b[36m{}\x1b[0m", info)
                } else {
                    info.to_string()
                }
            }
            OutputFormat::Json => serde_json::json!({
                "info": info,
                "timestamp": chrono::Utc::now().to_rfc3339()
            })
            .to_string(),
        }
    }

    fn colorize_response(&self, response: &str) -> String {
        // Simple colorization for common Minecraft server responses
        let mut colored = response.to_string();

        // Color player names (simple heuristic)
        if response.contains("players online:") {
            colored = colored.replace("players online:", "\x1b[32mplayers online:\x1b[0m");
        }

        // Color numbers
        colored = regex::Regex::new(r"\b(\d+)\b")
            .unwrap()
            .replace_all(&colored, "\x1b[33m$1\x1b[0m")
            .to_string();

        colored
    }
}
