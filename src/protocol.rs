use crate::error::{RconError, Result};
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::io::Cursor;

/// RCON packet types as defined in the protocol
pub mod packet_type {
    pub const AUTH: i32 = 3;
    pub const EXECCOMMAND: i32 = 2;
    pub const RESPONSE_VALUE: i32 = 0;
}

/// Maximum payload size for client-to-server packets
pub const MAX_REQUEST_PAYLOAD_SIZE: usize = 1446;

/// Maximum payload size for server-to-client packets
pub const MAX_RESPONSE_PAYLOAD_SIZE: usize = 4096;

/// Represents an RCON packet
#[derive(Debug, Clone)]
pub struct RconPacket {
    pub request_id: i32,
    pub packet_type: i32,
    pub payload: String,
}

impl RconPacket {
    /// Create a new RCON packet
    pub fn new(request_id: i32, packet_type: i32, payload: impl Into<String>) -> Self {
        Self {
            request_id,
            packet_type,
            payload: payload.into(),
        }
    }

    /// Create an authentication packet
    pub fn auth(request_id: i32, password: impl Into<String>) -> Self {
        Self::new(request_id, packet_type::AUTH, password)
    }

    /// Create a command execution packet
    pub fn command(request_id: i32, command: impl Into<String>) -> Self {
        Self::new(request_id, packet_type::EXECCOMMAND, command)
    }

    /// Serialize the packet to bytes
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        let payload_bytes = self.payload.as_bytes();

        // Check payload size limit
        if payload_bytes.len() > MAX_REQUEST_PAYLOAD_SIZE {
            return Err(RconError::InvalidPacket(format!(
                "Payload too large: {} bytes (max: {})",
                payload_bytes.len(),
                MAX_REQUEST_PAYLOAD_SIZE
            )));
        }

        // Calculate packet size: request_id + type + payload + 2 null bytes
        let packet_size = 4 + 4 + payload_bytes.len() + 2;

        let mut buffer = Vec::with_capacity(4 + packet_size);

        // Write packet length (excluding the length field itself)
        buffer
            .write_i32::<LittleEndian>(packet_size as i32)
            .map_err(|e| RconError::Protocol(format!("Failed to write packet length: {}", e)))?;

        // Write packet data
        buffer
            .write_i32::<LittleEndian>(self.request_id)
            .map_err(|e| RconError::Protocol(format!("Failed to write request ID: {}", e)))?;

        buffer
            .write_i32::<LittleEndian>(self.packet_type)
            .map_err(|e| RconError::Protocol(format!("Failed to write packet type: {}", e)))?;

        // Write payload and null terminators
        buffer.extend_from_slice(payload_bytes);
        buffer.push(0); // null terminator
        buffer.push(0); // padding

        Ok(buffer)
    }

    /// Deserialize a packet from bytes
    pub fn from_bytes(data: &[u8]) -> Result<Self> {
        if data.len() < 12 {
            return Err(RconError::InvalidPacket(
                "Packet too short (minimum 12 bytes required)".to_string(),
            ));
        }

        let mut cursor = Cursor::new(data);

        // Read packet length
        let packet_length = cursor.read_i32::<LittleEndian>().map_err(|e| {
            RconError::InvalidPacket(format!("Failed to read packet length: {}", e))
        })?;

        // Verify packet length matches data
        let expected_total_length = packet_length as usize + 4; // +4 for the length field itself
        if data.len() != expected_total_length {
            return Err(RconError::InvalidPacket(format!(
                "Packet length mismatch: expected {}, got {}",
                expected_total_length,
                data.len()
            )));
        }

        // Read packet data
        let request_id = cursor
            .read_i32::<LittleEndian>()
            .map_err(|e| RconError::InvalidPacket(format!("Failed to read request ID: {}", e)))?;

        let packet_type = cursor
            .read_i32::<LittleEndian>()
            .map_err(|e| RconError::InvalidPacket(format!("Failed to read packet type: {}", e)))?;

        // Read payload (everything except the last 2 null bytes)
        let payload_length = packet_length as usize - 8 - 2; // subtract request_id, type, and padding
        let mut payload_bytes = vec![0u8; payload_length];

        if payload_length > 0 {
            std::io::Read::read_exact(&mut cursor, &mut payload_bytes)
                .map_err(|e| RconError::InvalidPacket(format!("Failed to read payload: {}", e)))?;
        }

        // Convert payload to string, handling potential non-UTF8 bytes gracefully
        let payload = String::from_utf8_lossy(&payload_bytes)
            .trim_end_matches('\0')
            .to_string();

        Ok(Self {
            request_id,
            packet_type,
            payload,
        })
    }

    /// Check if this is an authentication response
    pub fn is_auth_response(&self) -> bool {
        self.packet_type == packet_type::EXECCOMMAND // Auth responses have type 2, not 3
    }

    /// Check if this is a command response
    pub fn is_command_response(&self) -> bool {
        self.packet_type == packet_type::RESPONSE_VALUE
    }

    /// Check if authentication was successful (for auth responses)
    pub fn auth_successful(&self, expected_request_id: i32) -> bool {
        self.is_auth_response() && self.request_id == expected_request_id
    }
}
