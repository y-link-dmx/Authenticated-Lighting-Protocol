use std::net::SocketAddr;
use std::time::{Duration, Instant};

use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use serde_cbor;
use thiserror::Error;
use tokio::net::UdpSocket;
use tokio::time::timeout;

use crate::discovery::DiscoveryClient as RawDiscoveryClient;
use crate::messages::{CapabilitySet, DeviceIdentity, DiscoveryReply, MessageType};

/// Represents a device observed during stateless discovery.
#[derive(Debug)]
pub struct DiscoveredDevice {
    pub addr: SocketAddr,
    pub identity: DeviceIdentity,
    pub capabilities: CapabilitySet,
    pub signed: bool,
}

/// Errors emitted by the SDK discovery helper.
#[derive(Debug, Error)]
pub enum DiscoveryClientError {
    #[error("io error: {0}")]
    Io(String),
    #[error("timeout waiting for replies")]
    Timeout,
    #[error("decode error: {0}")]
    Decode(String),
    #[error("unsupported version or message type")]
    UnsupportedVersion,
    #[error("signature invalid")]
    SignatureInvalid,
    #[error("discovery broadcast failed: {0}")]
    Broadcast(#[source] crate::discovery::DiscoveryError),
}

/// Stateless helper that wraps discovery broadcasts for SDK consumers.
#[derive(Debug, Clone)]
pub struct DiscoveryClient {
    local_addr: SocketAddr,
    broadcast_addr: SocketAddr,
    requested: Vec<String>,
    verifier: Option<VerifyingKey>,
    timeout: Duration,
}

impl DiscoveryClient {
    /// Create a client that scans for devices when `discover()` is invoked.
    pub fn new(
        local_addr: SocketAddr,
        broadcast_addr: SocketAddr,
        requested: Vec<String>,
        verifier: Option<VerifyingKey>,
        timeout: Duration,
    ) -> Self {
        Self {
            local_addr,
            broadcast_addr,
            requested,
            verifier,
            timeout,
        }
    }

    /// Broadcasts a discovery request and listens until the timeout.
    pub async fn discover(&self) -> Result<Vec<DiscoveredDevice>, DiscoveryClientError> {
        let socket = UdpSocket::bind(self.local_addr)
            .await
            .map_err(|e| DiscoveryClientError::Io(e.to_string()))?;
        socket
            .set_broadcast(true)
            .map_err(|e| DiscoveryClientError::Io(e.to_string()))?;

        let nonce = RawDiscoveryClient::broadcast(
            &socket,
            self.broadcast_addr,
            self.requested.clone(),
        )
        .await
        .map_err(DiscoveryClientError::Broadcast)?;

        let deadline = Instant::now() + self.timeout;
        let mut devices = Vec::new();
        let mut buffer = [0u8; 2048];

        loop {
            let remaining = deadline.saturating_duration_since(Instant::now());
            if remaining.is_zero() {
                break;
            }

            match timeout(remaining, socket.recv_from(&mut buffer)).await {
                Ok(Ok((len, addr))) => match serde_cbor::from_slice::<DiscoveryReply>(&buffer[..len]) {
                    Ok(reply) => match validate_reply(&reply, &nonce, self.verifier.as_ref()) {
                        Ok(signed) => {
                            devices.push(DiscoveredDevice {
                                addr,
                                identity: DeviceIdentity {
                                    device_id: reply.device_id.clone(),
                                    manufacturer_id: reply.manufacturer_id.clone(),
                                    model_id: reply.model_id.clone(),
                                    hardware_rev: reply.hardware_rev.clone(),
                                    firmware_rev: reply.firmware_rev.clone(),
                                },
                                capabilities: reply.capabilities.clone(),
                                signed,
                            });
                        }
                        Err(err) => return Err(err),
                    },
                    Err(err) => return Err(DiscoveryClientError::Decode(err.to_string())),
                },
                Ok(Err(err)) => return Err(DiscoveryClientError::Io(err.to_string())),
                Err(_) => break,
            }
        }

        if devices.is_empty() {
            return Err(DiscoveryClientError::Timeout);
        }

        Ok(devices)
    }
}

fn validate_reply(
    reply: &DiscoveryReply,
    expected_nonce: &[u8],
    verifier: Option<&VerifyingKey>,
) -> Result<bool, DiscoveryClientError> {
    if reply.message_type != MessageType::AlpineDiscoverReply
        || reply.alpine_version != crate::messages::ALPINE_VERSION
    {
        return Err(DiscoveryClientError::UnsupportedVersion);
    }

    if let Some(verifier) = verifier {
        let mut data = reply.server_nonce.clone();
        data.extend_from_slice(expected_nonce);
        let signature = Signature::from_slice(&reply.signature)
            .map_err(|_| DiscoveryClientError::SignatureInvalid)?;
        verifier
            .verify(&data, &signature)
            .map_err(|_| DiscoveryClientError::SignatureInvalid)?;
        Ok(true)
    } else {
        Ok(false)
    }
}
