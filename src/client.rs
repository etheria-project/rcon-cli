use crate::error::{RconError, Result};
use crate::protocol::{RconPacket, MAX_RESPONSE_PAYLOAD_SIZE};
use std::net::SocketAddr;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tracing::{debug, info, warn};

/// Configuration for RCON client connection
#[derive(Debug, Clone)]
pub struct RconConfig {
    pub address: SocketAddr,
    pub password: String,
    pub timeout: Duration,
}

impl RconConfig {
    pub fn new(address: SocketAddr, password: impl Into<String>) -> Self {
        Self {
            address,
            password: password.into(),
            timeout: Duration::from_secs(5),
        }
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }
}

/// RCON client for communicating with Minecraft servers
pub struct RconClient {
    stream: TcpStream,
    next_request_id: i32,
    config: RconConfig,
}

impl RconClient {
    /// Connect to an RCON server and authenticate
    pub async fn connect(config: RconConfig) -> Result<Self> {
        info!("Connecting to RCON server at {}", config.address);

        let stream = tokio::time::timeout(config.timeout, TcpStream::connect(config.address))
            .await
            .map_err(|_| RconError::Timeout)?
            .map_err(RconError::Network)?;

        let mut client = Self {
            stream,
            next_request_id: 1,
            config,
        };

        // Authenticate immediately after connection
        client.authenticate().await?;
        info!("Successfully connected and authenticated");

        Ok(client)
    }

    /// Authenticate with the server
    async fn authenticate(&mut self) -> Result<()> {
        debug!("Authenticating with server");

        let request_id = self.next_request_id();
        let auth_packet = RconPacket::auth(request_id, &self.config.password);

        self.send_packet(&auth_packet).await?;
        let response = self.read_packet().await?;

        if response.auth_successful(request_id) {
            debug!("Authentication successful");
            Ok(())
        } else {
            warn!("Authentication failed - invalid password or request ID mismatch");
            Err(RconError::AuthenticationFailed)
        }
    }

    /// Execute a command on the server
    pub async fn execute_command(&mut self, command: impl AsRef<str>) -> Result<String> {
        let command = command.as_ref();
        debug!("Executing command: {}", command);

        let request_id = self.next_request_id();
        let command_packet = RconPacket::command(request_id, command);

        self.send_packet(&command_packet).await?;

        // Handle potentially fragmented responses
        let response = self.read_command_response(request_id).await?;
        debug!(
            "Command executed successfully, response length: {} bytes",
            response.len()
        );

        Ok(response)
    }

    /// Test connectivity by sending a harmless command
    pub async fn ping(&mut self) -> Result<()> {
        debug!("Pinging server");
        let _ = self.execute_command("list").await?;
        debug!("Ping successful");
        Ok(())
    }

    /// Send a packet to the server
    async fn send_packet(&mut self, packet: &RconPacket) -> Result<()> {
        let bytes = packet.to_bytes()?;
        debug!(
            "Sending packet: type={}, id={}, size={} bytes",
            packet.packet_type,
            packet.request_id,
            bytes.len()
        );

        self.stream
            .write_all(&bytes)
            .await
            .map_err(RconError::Network)?;
        Ok(())
    }

    /// Read a single packet from the server
    async fn read_packet(&mut self) -> Result<RconPacket> {
        // Read packet length (4 bytes)
        let mut length_buffer = [0u8; 4];
        self.stream
            .read_exact(&mut length_buffer)
            .await
            .map_err(RconError::Network)?;

        let packet_length = i32::from_le_bytes(length_buffer) as usize;
        debug!("Reading packet of length: {} bytes", packet_length);

        // Validate packet length
        if packet_length < 8 {
            return Err(RconError::InvalidPacket(format!(
                "Packet too short: {} bytes",
                packet_length
            )));
        }

        if packet_length > MAX_RESPONSE_PAYLOAD_SIZE + 10 {
            return Err(RconError::InvalidPacket(format!(
                "Packet too large: {} bytes",
                packet_length
            )));
        }

        // Read the rest of the packet
        let mut packet_data = vec![0u8; packet_length + 4]; // +4 for length field
        packet_data[0..4].copy_from_slice(&length_buffer);

        self.stream
            .read_exact(&mut packet_data[4..])
            .await
            .map_err(RconError::Network)?;

        let packet = RconPacket::from_bytes(&packet_data)?;
        debug!(
            "Received packet: type={}, id={}, payload_len={}",
            packet.packet_type,
            packet.request_id,
            packet.payload.len()
        );

        Ok(packet)
    }

    /// Read command response, handling fragmentation
    async fn read_command_response(&mut self, expected_request_id: i32) -> Result<String> {
        let mut full_response = String::new();
        let mut packets_received = 0;

        loop {
            let packet = self.read_packet().await?;
            packets_received += 1;

            // Check if this packet belongs to our request
            if packet.request_id != expected_request_id {
                warn!(
                    "Received packet with unexpected request ID: {} (expected: {})",
                    packet.request_id, expected_request_id
                );
                continue;
            }

            // Check if this is a command response
            if !packet.is_command_response() {
                return Err(RconError::Protocol(format!(
                    "Expected command response, got packet type: {}",
                    packet.packet_type
                )));
            }

            full_response.push_str(&packet.payload);

            // Check if this is the last fragment
            // According to the spec, the last packet has payload < 4096 bytes
            if packet.payload.len() < MAX_RESPONSE_PAYLOAD_SIZE {
                debug!(
                    "Response complete after {} packet(s), total length: {} bytes",
                    packets_received,
                    full_response.len()
                );
                break;
            }

            // Safety check to prevent infinite loops
            if packets_received > 100 {
                return Err(RconError::Protocol(
                    "Too many response packets received".to_string(),
                ));
            }
        }

        Ok(full_response)
    }

    /// Generate the next request ID
    fn next_request_id(&mut self) -> i32 {
        let id = self.next_request_id;
        self.next_request_id = self.next_request_id.wrapping_add(1);
        if self.next_request_id == -1 {
            self.next_request_id = 1; // Skip -1 as it indicates auth failure
        }
        id
    }

    /// Get the server address this client is connected to
    pub fn server_address(&self) -> SocketAddr {
        self.config.address
    }

    /// Check if the connection is still alive
    pub async fn is_connected(&mut self) -> bool {
        // Try to send a minimal ping command
        self.ping().await.is_ok()
    }
}

/// Builder pattern for creating RCON client configurations
pub struct RconClientBuilder {
    address: Option<SocketAddr>,
    password: Option<String>,
    timeout: Duration,
}

impl RconClientBuilder {
    pub fn new() -> Self {
        Self {
            address: None,
            password: None,
            timeout: Duration::from_secs(5),
        }
    }

    pub fn address(mut self, address: SocketAddr) -> Self {
        self.address = Some(address);
        self
    }

    pub fn password(mut self, password: impl Into<String>) -> Self {
        self.password = Some(password.into());
        self
    }

    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    pub async fn connect(self) -> Result<RconClient> {
        let address = self
            .address
            .ok_or_else(|| RconError::InvalidConfig("Server address is required".to_string()))?;

        let password = self
            .password
            .ok_or_else(|| RconError::InvalidConfig("Password is required".to_string()))?;

        let config = RconConfig::new(address, password).with_timeout(self.timeout);
        RconClient::connect(config).await
    }
}

impl Default for RconClientBuilder {
    fn default() -> Self {
        Self::new()
    }
}
