use std::collections::HashMap;
use std::fmt;
use std::net::SocketAddr;
use std::net::UdpSocket as StdUdpSocket;
use std::sync::Arc;
use std::time::Duration;

use tokio::sync::Mutex;
use tokio::task::JoinHandle;

use crate::crypto::identity::NodeCredentials;
use crate::crypto::X25519KeyExchange;
use crate::control::{ControlClient, ControlCrypto};
use crate::handshake::keepalive;
use crate::handshake::transport::{CborUdpTransport, TimeoutTransport};
use crate::handshake::{HandshakeContext, HandshakeError};
use crate::messages::{CapabilitySet, ChannelFormat, ControlEnvelope, ControlOp, DeviceIdentity};
use crate::profile::{CompiledStreamProfile, StreamProfile};
use crate::session::AlnpSession;
use crate::stream::{AlnpStream, FrameTransport, StreamError};
use serde_json::Value;
use uuid::Uuid;

/// Errors emitted by the high-level SDK client.
///
/// These variants indicate what happened during discovery/handshake, streaming,
/// or UDP transport. They correspond to the guarantees documented on each method.
#[derive(Debug)]
#[non_exhaustive]
pub enum ClientError {
    /// OS-level failures such as socket bind/send errors or missing session data.
    Io(String),
    /// Handshake or session establishment failures propagated from `AlnpSession`.
    Handshake(HandshakeError),
    /// Streaming transport errors (e.g., `AlnpStream::send`).
    Stream(StreamError),
}


impl fmt::Display for ClientError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ClientError::Io(err) => write!(f, "io error: {}", err),
            ClientError::Handshake(err) => write!(f, "handshake error: {}", err),
            ClientError::Stream(err) => write!(f, "stream error: {}", err),
        }
    }
}

impl From<HandshakeError> for ClientError {
    fn from(err: HandshakeError) -> Self {
        ClientError::Handshake(err)
    }
}

impl From<StreamError> for ClientError {
    fn from(err: StreamError) -> Self {
        ClientError::Stream(err)
    }
}

impl From<std::io::Error> for ClientError {
    fn from(err: std::io::Error) -> Self {
        ClientError::Io(err.to_string())
    }
}

/// Thin UDP transport for the ALPINE streaming layer.
#[derive(Debug)]
struct UdpFrameTransport {
    socket: StdUdpSocket,
    peer: SocketAddr,
}

impl UdpFrameTransport {
    fn new(local: SocketAddr, peer: SocketAddr) -> Result<Self, std::io::Error> {
        let socket = StdUdpSocket::bind(local)?;
        socket.connect(peer)?;
        Ok(Self { socket, peer })
    }
}

impl FrameTransport for UdpFrameTransport {
    fn send_frame(&self, bytes: &[u8]) -> Result<(), String> {
        self.socket
            .send(bytes)
            .map_err(|e| format!("udp stream send: {}", e))?;
        Ok(())
    }
}

/// High-level controller client that orchestrates discovery, handshake, streaming,
/// control, and keepalive flows.
///
/// # Guarantees
/// * Handshake runs over `TimeoutTransport<CborUdpTransport>` and fails fast.
/// * Streaming uses a compiled `StreamProfile` and cannot change behavior once active.
/// * Keepalive tasks start after handshake and abort on `close()`.
#[derive(Debug)]
pub struct AlpineClient {
    session: AlnpSession,
    transport: Arc<Mutex<TimeoutTransport<CborUdpTransport>>>,
    local_addr: SocketAddr,
    remote_addr: SocketAddr,
    stream: Option<AlnpStream<UdpFrameTransport>>,
    control: ControlClient,
    keepalive_handle: Option<JoinHandle<()>>,
}

impl AlpineClient {
    /// Connects to a remote ALPINE device using the provided credentials.
    ///
    /// # Behavior
    /// * Executes discovery/handshake via `CborUdpTransport` and `TimeoutTransport`.
    /// * Spins up a keepalive future that ticks every 5 seconds.
    /// * Builds `ControlClient` once keys are derived so `control_envelope` works.
    ///
    /// # Errors
    /// Returns `ClientError::Io` for socket failures or missing session material,
    /// `ClientError::Handshake` for protocol errors, and `ClientError::Stream` for
    /// transport issues.
   pub async fn connect(
        local_addr: SocketAddr,
        remote_addr: SocketAddr,
        identity: DeviceIdentity,
        capabilities: CapabilitySet,
        credentials: NodeCredentials,
    ) -> Result<Self, ClientError> {
        let key_exchange = X25519KeyExchange::new();
        let authenticator = crate::session::Ed25519Authenticator::new(credentials.clone());

        let mut transport =
            TimeoutTransport::new(CborUdpTransport::bind(local_addr, remote_addr, 2048).await?, Duration::from_secs(3));
        let session = AlnpSession::connect(
            identity,
            capabilities.clone(),
            authenticator,
            key_exchange,
            HandshakeContext::default(),
            &mut transport,
        )
        .await?;

        let transport = Arc::new(Mutex::new(transport));
        let keepalive_handle = tokio::spawn(keepalive::spawn_keepalive(
            transport.clone(),
            Duration::from_secs(5),
            session
                .established()
                .ok_or_else(|| ClientError::Io("session missing after handshake".into()))?
                .session_id,
        ));

        let established = session
            .established()
            .ok_or_else(|| ClientError::Io("session missing after handshake".into()))?;
        let device_uuid = Uuid::parse_str(&established.device_identity.device_id)
            .unwrap_or_else(|_| Uuid::new_v4());
        let control_crypto = ControlCrypto::new(
            session
                .keys()
                .ok_or_else(|| ClientError::Io("session keys missing".into()))?,
        );
        let control = ControlClient::new(device_uuid, established.session_id, control_crypto);

        Ok(Self {
            session,
            transport,
            local_addr,
            remote_addr,
            stream: None,
            control,
            keepalive_handle: Some(keepalive_handle),
        })
    }

    /// Starts streaming with the selected profile; `Auto` is the default.
    ///
    /// # Guarantees
    /// * Profiles are validated/normalized; invalid combinations return explicit errors.
    /// * `config_id` is bound to the session and can't change once streaming begins.
    /// * Streaming transport is built after the profile is locked.
    ///
    /// # Errors
    /// Returns `ClientError::Io` for socket issues or session material that is missing.
    /// Returns `ClientError::Handshake` if the profile cannot be bound or the session rejects it.
    #[must_use]
    pub async fn start_stream(
        &mut self,
        profile: StreamProfile,
    ) -> Result<String, ClientError> {
        let compiled = profile
            .compile()
            .map_err(|err| HandshakeError::Protocol(err.to_string()))?;
        self.session
            .set_stream_profile(compiled.clone())
            .map_err(ClientError::Handshake)?;
        self.session.mark_streaming();

        let stream_socket = UdpFrameTransport::new(self.local_addr, self.remote_addr)?;
        let stream = AlnpStream::new(self.session.clone(), stream_socket, compiled.clone());
        self.stream = Some(stream);
        Ok(compiled.config_id().to_string())
    }

    /// Sends a streaming frame via the high-level helper.
    ///
    /// # Guarantees
    /// * Validation reuses `AlnpStream`, so it refuses to send when the session is not ready.
    /// * Applies jitter strategy before encoding.
    /// * Requires `start_stream` to have bound a profile before calling.
    ///
    /// # Errors
    /// Returns `StreamError` wrapped in `ClientError::Stream`.
    #[must_use]
    pub fn send_frame(
        &self,
        channel_format: ChannelFormat,
        channels: Vec<u16>,
        priority: u8,
        groups: Option<HashMap<String, Vec<u16>>>,
        metadata: Option<HashMap<String, serde_json::Value>>,
    ) -> Result<(), ClientError> {
        let stream = self
            .stream
            .as_ref()
            .ok_or_else(|| ClientError::Io("stream not started".into()))?;
        stream
            .send(channel_format, channels, priority, groups, metadata)
            .map_err(ClientError::from)
    }

    /// Gracefully closes the client, stopping keepalive tasks.
    ///
    /// # Behavior
    /// * Transitions the session state to closed.
    /// * Aborts the keepalive background job immediately.
    pub async fn close(mut self) {
        self.session.close();
        if let Some(handle) = self.keepalive_handle.take() {
            handle.abort();
        }
    }

    /// Builds an authenticated control envelope ready for transport.
    ///
    /// # Guarantees
    /// * Seals the payload with a MAC derived from the session keys.
    /// * Does not mutate transport state.
    ///
    /// # Errors
    /// Propagates the underlying `HandshakeError` returned while computing MACs.
    #[must_use]
    pub fn control_envelope(
        &self,
        seq: u64,
        op: ControlOp,
        payload: Value,
    ) -> Result<ControlEnvelope, HandshakeError> {
        self.control.envelope(seq, op, payload)
    }
}
