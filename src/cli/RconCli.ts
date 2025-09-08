import { RconClient } from "../client/RconClient.ts";
import type { RconConfig } from "../types/Packet.ts";

export interface CliArgs {
	host: string;
	port: number;
	password: string;
	command?: string;
	interactive?: boolean;
	help?: boolean;
}

export class RconCli {
	private client: RconClient;

	constructor(config: RconConfig) {
		this.client = new RconClient(config);
		this.setupEventHandlers();
	}

	static parseArgs(args: string[] = process.argv.slice(2)): CliArgs {
		const config: Partial<CliArgs> = {
			host: "localhost",
			port: 25575,
			interactive: false,
			help: false,
		};

		for (let i = 0; i < args.length; i++) {
			const arg = args[i];
			switch (arg) {
				case "-h":
				case "--host":
					config.host = args[++i];
					break;
				case "-p":
				case "--port":
					config.port = parseInt(args[++i], 10);
					if (Number.isNaN(config.port)) {
						throw new Error("Invalid port number");
					}
					break;
				case "-P":
				case "--password":
					config.password = args[++i];
					break;
				case "-c":
				case "--command":
					config.command = args[++i];
					break;
				case "-i":
				case "--interactive":
					config.interactive = true;
					break;
				case "--help":
					config.help = true;
					break;
				default:
					if (arg.startsWith("-")) {
						throw new Error(`Unknown option: ${arg}`);
					}
			}
		}

		return config as CliArgs;
	}

	static showHelp(): void {
		console.log(`
RCON Client - Minecraft Server Remote Console

Usage: bun run cli [options]

Options:
  -h, --host <host>         RCON server host (default: localhost)
  -p, --port <port>         RCON server port (default: 25575)
  -P, --password <password> RCON password (required)
  -c, --command <command>   Execute a single command
  -i, --interactive         Start interactive mode
  --help                    Show this help message

Examples:
  bun run cli -P mypass -c "list"
  bun run cli -P mypass -i
  bun run cli -h 192.168.1.100 -p 25575 -P mypass -c "time set day"

Interactive Commands:
  list                      List online players
  time set <time>           Set world time (day, night, 0-24000)
  weather <type>            Set weather (clear, rain, thunder)
  say <message>             Broadcast message to all players
  tp <player> <x> <y> <z>   Teleport player to coordinates
  gamemode <mode> <player>  Change player gamemode
  give <player> <item>      Give item to player
  kick <player>             Kick player from server
  exit, quit                Exit interactive mode
    `);
	}

	async run(args: CliArgs): Promise<void> {
		if (args.help) {
			RconCli.showHelp();
			return;
		}

		if (!args.password) {
			console.error("❌ Password is required. Use -P or --password");
			process.exit(1);
		}

		try {
			console.log(`🔌 Connecting to ${args.host}:${args.port}...`);
			await this.client.connect();

			console.log("🔐 Authenticating...");
			await this.client.authenticate();
			console.log("✅ Connected and authenticated successfully!\n");

			if (args.command) {
				await this.executeCommand(args.command);
			} else if (args.interactive) {
				await this.runInteractiveMode();
			} else {
				console.log(
					"ℹ️  No command specified. Use -c for single command or -i for interactive mode.",
				);
				RconCli.showHelp();
			}
		} catch (error) {
			console.error("❌ RCON Error:", (error as Error).message);
			process.exit(1);
		} finally {
			this.client.disconnect();
		}
	}

	private async executeCommand(command: string): Promise<void> {
		try {
			console.log(`> ${command}`);
			const response = await this.client.sendCommand(command);
			console.log(response || "(no response)");
		} catch (error) {
			console.error(
				`❌ Error executing command "${command}":`,
				(error as Error).message,
			);
		}
	}

	private async runInteractiveMode(): Promise<void> {
		console.log(
			'🎮 Interactive RCON mode. Type "help" for commands or "exit" to quit.\n',
		);

		// Setup readline-like interface using Bun
		process.stdin.setRawMode?.(false);
		process.stdout.write("> ");

		for await (const line of console) {
			const command = line.trim();

			if (command === "exit" || command === "quit") {
				console.log("👋 Goodbye!");
				break;
			}

			if (!command) {
				process.stdout.write("> ");
				continue;
			}

			if (command === "help") {
				this.showInteractiveHelp();
				process.stdout.write("> ");
				continue;
			}

			try {
				const response = await this.client.sendCommand(command);
				console.log(response || "(no response)");
			} catch (error) {
				console.error("❌ Error:", (error as Error).message);
			}

			process.stdout.write("> ");
		}
	}

	private showInteractiveHelp(): void {
		console.log(`
Common Minecraft Commands:
  list                      - List online players
  time set day             - Set time to day
  time set night           - Set time to night
  weather clear            - Clear weather
  weather rain             - Set rain
  say <message>            - Broadcast message
  tp <player> <x> <y> <z>  - Teleport player
  gamemode creative <player> - Set creative mode
  gamemode survival <player> - Set survival mode
  give <player> <item> <amount> - Give items

Server Commands:
  stop                     - Stop server
  save-all                 - Save world
  whitelist add <player>   - Add to whitelist
  whitelist remove <player> - Remove from whitelist
  kick <player> [reason]   - Kick player
  ban <player> [reason]    - Ban player

Type "exit" to quit interactive mode.
    `);
	}

	private setupEventHandlers(): void {
		this.client.on("connected", () => {
			// Already handled in run method
		});

		this.client.on("disconnected", () => {
			console.log("🔌 Disconnected from server");
		});

		this.client.on("authFailed", (reason) => {
			console.error("🔐 Authentication failed:", reason);
		});

		this.client.on("error", (error) => {
			console.error("❌ Connection error:", error.message);
		});
	}
}
