# RCON CLI

A robust command-line interface for communicating with Minecraft servers using the RCON (Remote Console) protocol.

## Features

- 🔌 **Full RCON Protocol Support** - Complete implementation of the Minecraft RCON protocol
- 🚀 **Async/Await** - Built with Tokio for efficient async networking
- 🎯 **Multiple Modes** - Single command execution, interactive sessions, and more
- 🎨 **Rich Output** - Colored output and multiple formatting options (text/JSON)
- 🔄 **Auto-Reconnection** - Automatic reconnection on connection loss
- 📦 **Response Fragmentation** - Proper handling of large server responses
- 🛡️ **Error Handling** - Comprehensive error handling and validation

## Installation

### From Source

```bash
git clone https://github.com/etheria-project/rcon-cli.git
cd rcon-cli
cargo build --release
```

The binary will be available at `target/release/rcon-cli`.

**Prerequisites:** Rust 1.70+ and a Minecraft server with RCON enabled.

## Quick Start

1. **Enable RCON** in your `server.properties`:
   ```properties
   enable-rcon=true
   rcon.password=your_password_here
   rcon.port=25575
   ```

2. **Execute commands**:
   ```bash
   # Single command
   ./rcon-cli -a localhost:25575 -p your_password exec "list"
   
   # Interactive session
   ./rcon-cli -a localhost:25575 -p your_password interactive
   ```

## Usage

```bash
rcon-cli [OPTIONS] <COMMAND>
```

### Global Options

- `-a, --address <HOST:PORT>` - Server address (default: localhost:25575)
- `-p, --password <PASSWORD>` - RCON password (or use RCON_PASSWORD env var)
- `-t, --timeout <SECONDS>` - Connection timeout (default: 5)
- `-v, --verbose` - Increase logging verbosity
- `-f, --format <FORMAT>` - Output format: text or json
- `--no-color` - Disable colored output

### Commands

#### Execute Single Command
```bash
rcon-cli -a localhost:25575 -p secret exec "list"
rcon-cli -a localhost:25575 -p secret exec --time "weather clear"
```

#### Interactive Mode
```bash
rcon-cli -a localhost:25575 -p secret interactive --prompt "minecraft> "
```

Interactive commands: `help`, `status`, `reconnect`, `quit`/`exit`

#### Additional Commands
```bash
# Test connectivity
rcon-cli -a localhost:25575 -p secret ping -c 5

# Server information
rcon-cli -a localhost:25575 -p secret info --detailed

# List players
rcon-cli -a localhost:25575 -p secret players --uuids
```

### Examples

#### Environment Variables & JSON Output
```bash
export RCON_PASSWORD="your_secret_password"
rcon-cli -a localhost:25575 -f json exec "list"
```

#### Common Minecraft Commands
```bash
# Player management
rcon-cli exec "kick player_name reason"
rcon-cli exec "gamemode creative player_name"

# World management
rcon-cli exec "time set day"
rcon-cli exec "weather clear"

# Server management
rcon-cli exec "save-all"
rcon-cli exec "whitelist add player_name"
```

## Library Usage

```rust
use rcon_cli::{RconClient, RconConfig};
use std::net::SocketAddr;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "localhost:25575".parse::<SocketAddr>()?;
    let config = RconConfig::new(addr, "my_password");
    
    let mut client = RconClient::connect(config).await?;
    let response = client.execute_command("list").await?;
    
    println!("Server response: {}", response);
    Ok(())
}
```

## Project Structure

```
src/
├── lib.rs          # Library root and public API
├── main.rs         # Binary entry point
├── cli.rs          # Command-line interface definitions
├── client.rs       # RCON client implementation
├── protocol.rs     # RCON protocol and packet handling
└── error.rs        # Error types and handling
```

## Releases

### Creating a New Release

1. Update `changelog.md` with your changes
2. Run the release script:
   - **Linux/macOS**: `./scripts/release.sh 1.1.0`
   - **Windows**: `scripts\release.bat 1.1.0`

When you push a version tag (`v*.*.*`), GitHub Actions automatically:
- Builds cross-platform binaries (Windows, Linux, macOS)
- Creates a GitHub release with changelog and downloadable archives

## Security Notes

⚠️ **Important:** RCON is not encrypted. Use strong passwords, limit to trusted networks, and consider SSH tunneling for remote access.

## Contributing

1. Fork the repository
2. Create a feature branch
3. Add tests for your changes
4. Submit a pull request

## License

MIT License - see the LICENSE file for details.

## Built from 💖 by Etheria

This project is crafted with love by the [Etheria](https://github.com/etheria-project) team. We're passionate about creating tools that make Minecraft server management easier and more enjoyable.

### Acknowledgments

- Built with [Clap](https://clap.rs/) for CLI parsing
- Uses [Tokio](https://tokio.rs/) for async runtime
- Implements the [Minecraft RCON Protocol](https://minecraft.wiki/w/RCON)
