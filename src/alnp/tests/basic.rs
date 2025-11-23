use std::sync::Mutex;
use std::time::Duration;

use alnp::handshake::{ChallengeAuthenticator, HandshakeError, HandshakeMessage, HandshakeTransport};
use alnp::handshake::transport::ReliableControlChannel;
use alnp::messages::{
    Acknowledge, CapabilitySet, ControlEnvelope, ControlHeader, ControlPayload, DeviceIdentity,
    IdentifyResponse, ProtocolVersion, SetMode,
};
use alnp::session::state::SessionState;
use alnp::session::{Ed25519Authenticator, StaticKeyAuthenticator};
use ed25519_dalek::SigningKey;
use rand::{rngs::OsRng, RngCore};
use tokio::time::sleep;
use uuid::Uuid;

struct AckTransport {
    ack: Mutex<Option<HandshakeMessage>>,
}

#[async_trait::async_trait]
impl HandshakeTransport for AckTransport {
    async fn send(&mut self, _msg: HandshakeMessage) -> Result<(), HandshakeError> {
        Ok(())
    }

    async fn recv(&mut self) -> Result<HandshakeMessage, HandshakeError> {
        loop {
            if let Some(msg) = self.ack.lock().unwrap().take() {
                return Ok(msg);
            }
            sleep(Duration::from_millis(10)).await;
        }
    }
}

#[tokio::test]
async fn x25519_round_trip_placeholder() {
    // Ensure reliable channel handles ack and sequence propagation.
    let ack_header = ControlHeader {
        seq: 1,
        nonce: vec![1, 2, 3],
        timestamp_ms: 0,
    };
    let ack = Acknowledge {
        header: ack_header.clone(),
        ok: true,
        detail: None,
        signature: vec![],
    };

    let mut channel = ReliableControlChannel::new(AckTransport {
        ack: Mutex::new(Some(HandshakeMessage::Ack(ack))),
    });

    let envelope = ControlEnvelope {
        header: ControlHeader {
            seq: 0,
            nonce: vec![],
            timestamp_ms: 0,
        },
        payload: ControlPayload::IdentifyResponse(IdentifyResponse {
            acknowledged: true,
            detail: None,
        }),
        signature: vec![],
    };

    let result = channel.send_reliable(envelope).await;
    assert!(result.is_ok());
}

#[test]
fn ed25519_signature_roundtrip() {
    let mut secret = [0u8; 32];
    OsRng.fill_bytes(&mut secret);
    let signing = SigningKey::from_bytes(&secret);
    let creds = alnp::crypto::identity::NodeCredentials {
        signing: signing.clone(),
        verifying: signing.verifying_key(),
    };
    let auth = Ed25519Authenticator::new(creds);
    let nonce = b"hello-nonce";
    let sig = auth.sign_challenge(nonce);
    assert!(auth.verify_challenge(nonce, &sig));
}

#[test]
fn session_state_transitions() {
    let init = SessionState::Init;
    let handshake = init.transition(SessionState::Handshake).unwrap();
    let auth = handshake
        .transition(SessionState::Authenticated {
            since: std::time::Instant::now(),
        })
        .unwrap();
    let ready = auth
        .transition(SessionState::Ready {
            since: std::time::Instant::now(),
        })
        .unwrap();
    assert!(ready
        .transition(SessionState::Streaming {
            since: std::time::Instant::now()
        })
        .is_ok());
}

#[test]
fn static_key_authenticator_roundtrip() {
    let auth = StaticKeyAuthenticator::default();
    let nonce = b"bytes";
    let sig = auth.sign_challenge(nonce);
    assert!(auth.verify_challenge(nonce, &sig));
}

#[test]
fn capability_set_serializes() {
    let caps = CapabilitySet {
        supports_encryption: true,
        supports_redundancy: true,
        max_universes: Some(10),
        vendor_data: Some("test".to_string()),
    };
    let json = serde_json::to_string(&caps).unwrap();
    assert!(json.contains("supports_encryption"));
}

#[test]
fn device_identity_helper() {
    let ident = DeviceIdentity {
        cid: Uuid::new_v4(),
        manufacturer: "ALNP".into(),
        model: "Y1".into(),
        firmware_rev: "1.0.0".into(),
    };
    let info = alnp::messages::DeviceInfo {
        identity: ident,
        version: ProtocolVersion::alnp_v1(),
        mode: alnp::messages::OperatingMode::Normal,
    };
    assert_eq!(info.version.major, 1);
}

#[test]
fn control_envelope_signature_roundtrip() {
    use alnp::control::{ControlCrypto, ControlResponder};
    let mut secret = [0u8; 32];
    OsRng.fill_bytes(&mut secret);
    let signing = SigningKey::from_bytes(&secret);
    let verifier = signing.verifying_key();
    let crypto = ControlCrypto::new(signing, Some(verifier));
    let payload = ControlPayload::SetMode(SetMode {
        mode: alnp::messages::OperatingMode::Normal,
    });
    let env = crypto.sign_envelope(ControlEnvelope {
        header: ControlHeader {
            seq: 1,
            nonce: vec![9, 9, 9],
            timestamp_ms: 1,
        },
        payload,
        signature: vec![],
    });
    let responder = ControlResponder::new(
        DeviceIdentity {
            cid: Uuid::new_v4(),
            manufacturer: "ALNP".into(),
            model: "Y1".into(),
            firmware_rev: "1.0".into(),
        },
        crypto,
    );
    assert!(responder.verify(&env).is_ok());
}
