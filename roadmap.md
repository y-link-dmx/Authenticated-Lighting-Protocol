# ALPINE Roadmap
*Authenticated Lighting Protocol*

This roadmap shows where ALPINE is heading, which release will carry each capability, and which phases are already behind us.

---

## Phase 1 – Core Foundations (v1.0, completed)
**Status:** ✅ Complete

**Goal:** Deliver a rock-solid baseline that works on Ethernet and WiFi without special configuration.

- Finalized the v1 discovery, handshake, control, and streaming wire formats.
- Documented loss handling, jitter recovery, and deterministic failure behavior.
- Published SDK-friendly bindings so `Auto` is the default safe mode for most users.

**Outcome:**  
Real deployments now rely on ALPINE v1 with predictable behavior.

---

## Phase 2 – Stream Profiles & Selectable Behavior (target v1.2)
**Status:** In progress

**Goal:** Let users choose between safe defaults (Auto), low latency (Realtime), or install-friendly behavior without compromising guarantees.

- Introduce stream profiles as first-class objects with validation and immutable config IDs.
- Bind profile identity to the session to prevent unsafe runtime swaps.
- Provide deterministic fallbacks when profiles conflict.

**Outcome:**  
Operators select predictable behavior tailored to their venue.

---

## Phase 3 – Adaptive Streaming & Network Resilience (target v1.3)
**Status:** Planned

**Goal:** Keep ALPINE stable even when packet loss, jitter, or late frames appear.

- Automatically detect loss, gaps, and jitter and adjust keyframe cadence, delta encoding, and deadlines.
- Force recovery keyframes and optionally smooth/predict on devices.
- Provide observability so users understand why quality shifted.

**Outcome:**  
The protocol degrades gracefully while preserving temporal correctness.

---

## Phase 4 – Custom Profiles & Preferences (target v1.4)
**Status:** Planned

**Goal:** Let advanced users express latency/smoothness/resilience preferences without exposing low-level flags.

- Allow naming, validating, and compiling custom profiles expressed as high-level goals.
- Reject unsafe combinations before they hit the wire; provide clear validation errors.
- Allow sharing profiles across teams.

**Outcome:**  
Power users get control while the runtime remains deterministic.

---

## Phase 5 – Security & Trust Hardening (target v1.5)
**Status:** Planned

**Goal:** Harden identities, replay protection, and optional encryption without adding gimmicks.

- Certificate-backed identities and session binding.
- Replay protection across restarts and optional encrypted payloads for high-security installs.
- Clear security documentation and conservative defaults.

**Outcome:**  
Security is built in, not bolted on.

---

## Phase 6 – SDKs, Tooling & Developer Experience (v1.11 completed)
**Status:** ✅ Complete

**Goal:** Make ALPINE the easiest protocol to adopt via SDKs and documentation.

- Added SDK layers for Rust (`src/alnp/src/sdk`), TypeScript (`bindings/ts/src/sdk`), Python (`bindings/python/src/alnp/sdk`), and C++ (`bindings/cpp/sdk/alpine_sdk.hpp`) so developers can call `connect()`, `send_frame()`, `control()`, and keepalive helpers.
- Position SDKs as the recommended entry points in the README/docs while keeping bindings stable for constrained environments.
- Embedded validation, docs packaging, and GHCR C packages continue to accompany each release.

**Outcome:**  
App developers rely on SDK helpers while embedded or low-level teams interact with the stable bindings.

---

## Phase 7 – Ecosystem Growth & Compatibility (target v2.0)
**Status:** Planned

**Goal:** Expand ALPINE safely as the platform grows.

- Introduce capability negotiation and vendor-defined extension ranges.
- Keep strict backward compatibility guarantees.
- Establish clean upgrade paths for future hardware and software.

**Outcome:**  
ALPINE becomes a stable foundation everyone can build on.

---

## Design Commitment

> **Under packet loss, jitter, or delay, ALPINE degrades visual quality—never temporal correctness.**

This principle guides every phase.
