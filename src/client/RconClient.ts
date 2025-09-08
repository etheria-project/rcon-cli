import { EventEmitter } from "../lib/EventEmitter.ts";
import { createPacket, parsePacket } from "../lib/PacketBuilder.ts";
import type { RconClientEvents, RconConfig } from "../types/Packet.ts";
import { RconPacketType } from "../types/Packet.ts";

export class RconClient extends EventEmitter<RconClientEvents> {
	private socket: {
		end(): void;
		write(data: Buffer): void;
	} | null = null;
	private config: Required<RconConfig>;
	private isAuthenticated = false;
	private requestId = 0;
	private pendingRequests = new Map<
		number,
		{
			resolve: (response: string) => void;
			reject: (error: Error) => void;
			timer: Timer;
		}
	>();
	private responseBuffer = new Map<number, string[]>();

	constructor(config: RconConfig) {
		super();
		this.config = {
			timeout: 5000,
			...config,
		};
	}

	/**
	 * Connect to the RCON server
	 */
	async connect(): Promise<void> {
		if (this.socket) {
			throw new Error("Already connected");
		}

		return new Promise((resolve, reject) => {
			try {
				Bun.connect({
					hostname: this.config.host,
					port: this.config.port,
					socket: {
						data: (_socket, data) => {
							this.handleData(data);
						},
						open: (socket) => {
							console.log(
								`Connected to RCON server at ${this.config.host}:${this.config.port}`,
							);
							// Store the socket reference for later use
							this.socket = {
								end: () => socket.end(),
								write: (data: Buffer) => socket.write(data),
							};
							this.emit("connected");
							resolve();
						},
						close: (_socket) => {
							console.log("RCON connection closed");
							this.handleDisconnection();
						},
						error: (_socket, error) => {
							console.error("RCON connection error:", error);
							this.emit("error", error);
							reject(error);
						},
					},
				});
			} catch (error) {
				reject(error);
			}
		});
	}

	/**
	 * Authenticate with the RCON server
	 */
	async authenticate(): Promise<boolean> {
		if (this.isAuthenticated) {
			return true;
		}

		if (!this.socket) {
			throw new Error("Not connected. Call connect() first.");
		}

		return new Promise((resolve, reject) => {
			const requestId = this.getNextRequestId();
			const packet = createPacket(
				requestId,
				RconPacketType.SERVERDATA_AUTH,
				this.config.password,
			);

			const timer = setTimeout(() => {
				this.cleanupRequest(requestId);
				const error = new Error("Authentication timeout");
				this.emit("authFailed", error.message);
				reject(error);
			}, this.config.timeout);

			this.pendingRequests.set(requestId, {
				resolve: (_response) => {
					this.isAuthenticated = true;
					console.log("Successfully authenticated with RCON server");
					this.emit("authenticated");
					resolve(true);
				},
				reject,
				timer,
			});

			this.sendPacket(packet);
		});
	}

	/**
	 * Send a command to the server
	 */
	async sendCommand(command: string): Promise<string> {
		if (!this.isAuthenticated) {
			throw new Error("Not authenticated. Call authenticate() first.");
		}

		if (!this.socket) {
			throw new Error("Not connected.");
		}

		return new Promise((resolve, reject) => {
			const requestId = this.getNextRequestId();
			const packet = createPacket(
				requestId,
				RconPacketType.SERVERDATA_EXECCOMMAND,
				command,
			);

			const timer = setTimeout(() => {
				this.cleanupRequest(requestId);
				reject(new Error("Command execution timeout"));
			}, this.config.timeout);

			this.pendingRequests.set(requestId, {
				resolve: (response) => {
					resolve(response.trim());
				},
				reject,
				timer,
			});

			this.sendPacket(packet);
		});
	}

	/**
	 * Disconnect from the server
	 */
	disconnect(): void {
		if (this.socket) {
			this.socket.end();
			this.socket = null;
		}
		this.handleDisconnection();
	}

	/**
	 * Check if client is connected
	 */
	isConnected(): boolean {
		return this.socket !== null;
	}

	/**
	 * Check if client is authenticated
	 */
	isAuth(): boolean {
		return this.isAuthenticated;
	}

	private handleData(data: Buffer): void {
		const packet = parsePacket(data);
		if (!packet) {
			console.error("Failed to parse RCON packet");
			return;
		}

		// Handle authentication failure
		if (packet.requestId === -1) {
			const error = "RCON authentication failed: Invalid password";
			console.error(error);
			this.emit("authFailed", error);
			this.cleanupAllRequests(new Error(error));
			return;
		}

		// Handle pending requests
		const request = this.pendingRequests.get(packet.requestId);
		if (request) {
			clearTimeout(request.timer);

			// Handle fragmented responses
			if (packet.type === RconPacketType.SERVERDATA_RESPONSE_VALUE) {
				if (!this.responseBuffer.has(packet.requestId)) {
					this.responseBuffer.set(packet.requestId, []);
				}

				this.responseBuffer.get(packet.requestId)?.push(packet.payload);

				// Check if this might be the end of a fragmented response
				// We'll send a dummy command to detect the end
				if (packet.payload.length === 4096) {
					// Likely fragmented, wait for more
					return;
				}

				// Complete response
				const fullResponse = this.responseBuffer
					.get(packet.requestId)
					?.join("");
				this.responseBuffer.delete(packet.requestId);
				this.pendingRequests.delete(packet.requestId);
				request.resolve(fullResponse ?? "");
			} else {
				this.pendingRequests.delete(packet.requestId);
				request.resolve(packet.payload);
			}
		}
	}

	private handleDisconnection(): void {
		this.isAuthenticated = false;
		this.socket = null;
		this.cleanupAllRequests(new Error("Connection closed"));
		this.emit("disconnected");
	}

	private sendPacket(packet: Buffer): void {
		if (this.socket) {
			this.socket.write(packet);
		}
	}

	private getNextRequestId(): number {
		return ++this.requestId;
	}

	private cleanupRequest(requestId: number): void {
		const request = this.pendingRequests.get(requestId);
		if (request) {
			clearTimeout(request.timer);
			this.pendingRequests.delete(requestId);
		}
		this.responseBuffer.delete(requestId);
	}

	private cleanupAllRequests(error: Error): void {
		for (const [_requestId, request] of this.pendingRequests) {
			clearTimeout(request.timer);
			request.reject(error);
		}
		this.pendingRequests.clear();
		this.responseBuffer.clear();
	}
}
