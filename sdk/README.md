# ALPINE SDK

This directory houses the `alpine-protocol-sdk` Rust crate and future SDKs for other languages. Each SDK depends on the published bindings (`alpine-protocol-rs`, `@alpine-core/protocol`, `alpine-protocol`), uses only their public APIs, and wraps them with more ergonomic abstractions.

For now, the Rust SDK exposes `AlpineSdkClient` that orchestrates discovery, handshake, and streaming on top of the `alpine-protocol-rs::sdk::AlpineClient`.
