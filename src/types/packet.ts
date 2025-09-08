export interface RconPacket {
	length: number;
	requestId: number;
	type: RconPacketType;
	payload: string;
}

export interface RconConfig {
	host: string;
	port: number;
	password: string;
	timeout?: number;
}

export interface RconResponse {
	requestId: number;
	payload: string;
	isComplete: boolean;
}

export enum RconPacketType {
	SERVERDATA_AUTH = 3,
	SERVERDATA_EXECCOMMAND = 2,
	SERVERDATA_RESPONSE_VALUE = 0,
	SERVERDATA_AUTH_RESPONSE = 2,
}

export interface RconClientEvents extends Record<string, unknown> {
	connected: () => void;
	disconnected: () => void;
	authenticated: () => void;
	authFailed: (reason: string) => void;
	error: (error: Error) => void;
}
