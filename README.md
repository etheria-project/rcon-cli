# 🎮 Minecraft RCON Client

A robust, TypeScript-based RCON (Remote Console) client for Minecraft servers built with [Bun](https://bun.sh). Features a clean modular architecture, full type safety, and both CLI and programmatic interfaces.

## 🏗️ Architecture

```
src/
├── types/
│   └── Packet.ts          # TypeScript interfaces and enums
├── lib/
│   ├── PacketBuilder.ts   # RCON packet creation and parsing utilities
│   └── EventEmitter.ts    # Custom event emitter for client events
├── client/
│   └── RconClient.ts      # Main RCON client class
├── cli/
│   ├── RconCli.ts         # CLI interface class
│   └── index.ts           # CLI entry point
└── index.ts               # Main entry point with exports
```

## ✨ Features

- **🔧 Clean Architecture** - Modular design with separation of concerns
- **📘 Full TypeScript** - Complete type safety with comprehensive interfaces
- **🎯 RCON Protocol Compliant** - Implements the full [Minecraft RCON specification](https://minecraft.wiki/w/RCON)
- **⚡ Bun Native** - Uses Bun's built-in TCP support for optimal performance
- **🎮 CLI Interface** - Easy-to-use command line tool with interactive mode
- **📚 Library API** - Clean programmatic interface for integration
- **🔗 Event-Driven** - Event emitter pattern for loose coupling
- **📦 Fragmentation Handling** - Properly handles large responses split across multiple packets
- **🔐 Authentication** - Secure password-based authentication
- **⏱️ Timeout Management** - Configurable timeouts with proper cleanup

## 📦 Installation

```bash
# Clone or copy the project
git clone <your-repo>
cd rcon

# Install dependencies
bun install
```

## 🚀 Usage

### CLI Usage

```bash
# Single command execution
bun run cli -P your_password -c "list"
bun run cli -P your_password -c "time set day"
bun run cli -P your_password -c "weather clear"

# Interactive mode
bun run cli -P your_password -i

# Custom server connection
bun run cli -h 192.168.1.100 -p 25575 -P your_password -c "list"

# Show help
bun run cli:help
```

#### CLI Options

| Option | Description | Default |
|--------|-------------|---------|
| `-h, --host` | RCON server host | `localhost` |
| `-p, --port` | RCON server port | `25575` |
| `-P, --password` | RCON password | *(required)* |
| `-c, --command` | Execute single command | - |
| `-i, --interactive` | Start interactive mode | `false` |
| `--help` | Show help message | - |

### Programmatic Usage

```typescript
import { RconClient, RconConfig } from './src/index.ts';

// Basic usage
const client = new RconClient({
  host: 'localhost',
  port: 25575,
  password: 'your_rcon_password',
  timeout: 5000 // optional, default: 5000ms
});

try {
  // Connect and authenticate
  await client.connect();
  await client.authenticate();

  // Execute commands
  const players = await client.sendCommand('list');
  console.log('Online players:', players);

  const time = await client.sendCommand('time query daytime');
  console.log('Current time:', time);

  // Broadcast message
  await client.sendCommand('say Hello from RCON!');

} catch (error) {
  console.error('RCON Error:', error);
} finally {
  client.disconnect();
}
```

#### Event Handling

```typescript
import { RconClient } from './src/index.ts';

const client = new RconClient(config);

// Set up event listeners
client.on('connected', () => {
  console.log('🔌 Connected to server');
});

client.on('authenticated', () => {
  console.log('🔐 Successfully authenticated');
});

client.on('disconnected', () => {
  console.log('📤 Disconnected from server');
});

client.on('authFailed', (reason) => {
  console.error('❌ Authentication failed:', reason);
});

client.on('error', (error) => {
  console.error('💥 Connection error:', error);
});
```

#### Advanced Usage

```typescript
import { RconClient, PacketBuilder, RconPacketType } from './src/index.ts';

// Check connection status
if (client.isConnected() && client.isAuth()) {
  const response = await client.sendCommand('your_command');
}

// Use packet builder directly (advanced)
const packet = PacketBuilder.createPacket(
  1, 
  RconPacketType.SERVERDATA_EXECCOMMAND, 
  'list'
);
```

## 🎮 Common Minecraft Commands

| Command | Description |
|---------|-------------|
| `list` | List online players |
| `time set day` | Set time to day |
| `time set night` | Set time to night |
| `time query daytime` | Get current game time |
| `weather clear` | Clear weather |
| `weather rain` | Set rain |
| `weather thunder` | Set thunderstorm |
| `say <message>` | Broadcast message to all players |
| `tp <player> <x> <y> <z>` | Teleport player to coordinates |
| `gamemode creative <player>` | Set player to creative mode |
| `gamemode survival <player>` | Set player to survival mode |
| `give <player> <item> <amount>` | Give items to player |
| `kick <player> [reason]` | Kick player from server |
| `ban <player> [reason]` | Ban player from server |
| `whitelist add <player>` | Add player to whitelist |
| `stop` | Stop the server |
| `save-all` | Save the world |

## ⚙️ Server Configuration

To enable RCON on your Minecraft server, edit `server.properties`:

```properties
enable-rcon=true
rcon.password=your_secure_password
rcon.port=25575
broadcast-rcon-to-ops=false
```

> ⚠️ **Security Warning**: Never expose RCON ports to the internet. RCON is not encrypted and can be subject to man-in-the-middle attacks.

## 🔧 Development

```bash
# Development mode (with file watching)
bun run dev

# Linting
bun run lint

# Formatting
bun run format
```

## 📁 Project Structure

```
rcon/
├── src/
│   ├── types/           # TypeScript type definitions
│   ├── lib/             # Utility libraries
│   ├── client/          # Core RCON client
│   ├── cli/             # Command-line interface
│   └── index.ts         # Main exports
├── package.json         # Project configuration
├── tsconfig.json        # TypeScript configuration
├── biome.json          # Biome linter/formatter config
└── README.md           # This file
```

## 🎯 API Reference

### RconClient

```typescript
class RconClient extends EventEmitter<RconClientEvents>
```

#### Methods

- `connect(): Promise<void>` - Connect to RCON server
- `authenticate(): Promise<boolean>` - Authenticate with password
- `sendCommand(command: string): Promise<string>` - Send command and get response
- `disconnect(): void` - Disconnect from server
- `isConnected(): boolean` - Check connection status
- `isAuth(): boolean` - Check authentication status

#### Events

- `connected` - Fired when connected to server
- `authenticated` - Fired when successfully authenticated
- `disconnected` - Fired when disconnected from server
- `authFailed` - Fired when authentication fails
- `error` - Fired on connection errors

### Types

```typescript
interface RconConfig {
  host: string;
  port: number;
  password: string;
  timeout?: number; // default: 5000ms
}

interface RconPacket {
  length: number;
  requestId: number;
  type: RconPacketType;
  payload: string;
}

enum RconPacketType {
  SERVERDATA_AUTH = 3,
  SERVERDATA_EXECCOMMAND = 2,
  SERVERDATA_RESPONSE_VALUE = 0,
  SERVERDATA_AUTH_RESPONSE = 2
}
```

## 📚 References

- [Minecraft RCON Protocol Documentation](https://minecraft.wiki/w/RCON)
- [Bun Documentation](https://bun.sh/docs)
- [TypeScript Documentation](https://www.typescriptlang.org/docs)

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🤝 Contributing

1. Fork the project
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

---

Built with ❤️ using [Bun](https://bun.sh) and TypeScript
