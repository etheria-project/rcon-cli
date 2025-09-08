import type { RconPacket, RconPacketType } from "../types/Packet.ts";

/**
 * Creates an RCON packet buffer
 */
export function createPacket(
	requestId: number,
	type: RconPacketType,
	payload: string,
): Buffer {
	const payloadBuffer = Buffer.from(payload, "ascii");
	const length = 4 + 4 + payloadBuffer.length + 2; // requestId + type + payload + 2 null bytes

	const packet = Buffer.allocUnsafe(4 + length);
	let offset = 0;

	// Write length (little-endian)
	packet.writeInt32LE(length, offset);
	offset += 4;

	// Write request ID (little-endian)
	packet.writeInt32LE(requestId, offset);
	offset += 4;

	// Write type (little-endian)
	packet.writeInt32LE(type, offset);
	offset += 4;

	// Write payload
	payloadBuffer.copy(packet, offset);
	offset += payloadBuffer.length;

	// Write null terminators
	packet.writeUInt8(0, offset);
	packet.writeUInt8(0, offset + 1);

	return packet;
}

/**
 * Parses an RCON packet from buffer
 */
export function parsePacket(buffer: Buffer): RconPacket | null {
	if (buffer.length < 12) {
		return null; // Minimum packet size
	}

	let offset = 0;

	// Read length
	const length = buffer.readInt32LE(offset);
	offset += 4;

	if (buffer.length < 4 + length) {
		return null; // Incomplete packet
	}

	// Read request ID
	const requestId = buffer.readInt32LE(offset);
	offset += 4;

	// Read type
	const type = buffer.readInt32LE(offset) as RconPacketType;
	offset += 4;

	// Read payload (exclude the 2 null bytes at the end)
	const payloadLength = length - 4 - 4 - 2;
	const payload = buffer
		.subarray(offset, offset + payloadLength)
		.toString("ascii");

	return { length, requestId, type, payload };
}

/**
 * Validates if a packet type is valid
 */
export function isValidPacketType(type: number): type is RconPacketType {
	return Object.values(RconPacketType).includes(type as RconPacketType);
}
