//! SDK helpers that wrap the low-level ALPINE bindings with ergonomic clients.
//! 
//! This module provides the authoritative reference for higher-level behavior,
//! including the discovery → handshake → stream lifecycle, keepalive management,
//! and control envelope helpers. Documented guarantees here are canonical.
pub mod client;

pub use client::{AlpineClient, ClientError};
