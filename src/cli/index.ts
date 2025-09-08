#!/usr/bin/env bun

import { RconCli } from "./RconCli.ts";

async function main() {
	try {
		const args = RconCli.parseArgs();
		const cli = new RconCli({
			host: args.host,
			port: args.port,
			password: args.password,
		});

		await cli.run(args);
	} catch (error) {
		if (error instanceof Error) {
			console.error("❌", error.message);
			if (
				error.message.includes("Unknown option") ||
				error.message.includes("Invalid")
			) {
				console.log("\nUse --help for usage information.");
			}
		} else {
			console.error("❌ Unexpected error:", error);
		}
		process.exit(1);
	}
}

if (import.meta.main) {
	main().catch(console.error);
}
