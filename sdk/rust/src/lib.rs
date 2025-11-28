//! High-level Rust SDK built on top of the bindings.
use alpine_protocol_rs::profile::{StreamProfile, CompiledStreamProfile};
use alpine_protocol_rs::sdk::{AlpineClient as BindingsClient, ClientError as BindingsError};

/// SDK error wraps the binding's client error but keeps the layering clear.
pub type SDKError = BindingsError;

/// SDK client exposes a simplified lifecycle on top of `alpine-protocol-rs`.
pub struct AlpineSdkClient {
    inner: BindingsClient,
}

impl AlpineSdkClient {
    /// Builds a new SDK client using the binding's `AlpineClient`.
    pub async fn connect(
        local_addr: std::net::SocketAddr,
        remote_addr: std::net::SocketAddr,
        identity: alpine_protocol_rs::messages::DeviceIdentity,
        capabilities: alpine_protocol_rs::messages::CapabilitySet,
        credentials: alpine_protocol_rs::crypto::identity::NodeCredentials,
    ) -> Result<Self, SDKError> {
        let inner = BindingsClient::connect(
            local_addr,
            remote_addr,
            identity,
            capabilities,
            credentials,
        )
        .await?;
        Ok(Self { inner })
    }

    /// Starts streaming with a declarative stream profile.
    pub async fn start_stream(
        &mut self,
        profile: StreamProfile,
    ) -> Result<String, SDKError> {
        self.inner.start_stream(profile).await
    }
}
