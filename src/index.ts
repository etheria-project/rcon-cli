// Main exports

export { RconCli } from "./cli/RconCli.ts";
export { RconClient } from "./client/RconClient.ts";
export { EventEmitter } from "./lib/EventEmitter.ts";
export {
	createPacket,
	isValidPacketType,
	parsePacket,
} from "./lib/PacketBuilder.ts";

// Type exports
export type {
	RconClientEvents,
	RconConfig,
	RconPacket,
	RconResponse,
} from "./types/Packet.ts";
export { RconPacketType } from "./types/Packet.ts";

// Example usage
async function _exampleUsage() {
	const { RconClient } = await import("./client/RconClient.ts");

	const client = new RconClient({
		host: "localhost",
		port: 25575,
		password: "your_password",
	});

	try {
		// Connect and authenticate
		await client.connect();
		await client.authenticate();

		// Execute commands
		const players = await client.sendCommand("list");
		console.log("Online players:", players);

		const time = await client.sendCommand("time query daytime");
		console.log("Current time:", time);

		// Set up event listeners
		client.on("disconnected", () => {
			console.log("Server disconnected");
		});
	} catch (error) {
		console.error("RCON Error:", error);
	} finally {
		client.disconnect();
	}
}

// Run example if this file is executed directly
if (import.meta.main) {
	console.log("🎮 Minecraft RCON Client");
	console.log("========================");
	console.log("");
	console.log("This is a TypeScript RCON client for Minecraft servers.");
	console.log("");
	console.log("Usage examples:");
	console.log("");
	console.log("CLI Usage:");
	console.log('  bun run src/cli/index.ts -P password -c "list"');
	console.log("  bun run src/cli/index.ts -P password -i");
	console.log("");
	console.log("Programmatic Usage:");
	console.log('  import { RconClient } from "./src/index.ts";');
	console.log("");
	console.log("For more information, see the README or use --help");
}
