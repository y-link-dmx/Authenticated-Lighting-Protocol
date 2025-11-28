# ALPINE — Authenticated Lighting Protocol (v1.0)

[![Rust](https://img.shields.io/badge/Rust-crates.io-000000?style=for-the-badge&logo=rust&logoColor=white)](https://crates.io/crates/alpine-protocol-rs)
[![Python](https://img.shields.io/badge/Python-PyPI-3776AB?style=for-the-badge&logo=python&logoColor=white)](https://pypi.org/project/alnp/)
[![C](https://img.shields.io/badge/C-GitHub%20Packages-181717?style=for-the-badge&logo=github&logoColor=white)](https://github.com/orgs/alpine-core/packages?tab=packages&q=alnp-c)
[![License](https://img.shields.io/badge/License-Apache--2.0-blue?style=for-the-badge)](LICENSE)

---
ALPINE is a **modern, secure, vendor-agnostic lighting control protocol** designed to replace legacy systems such as sACN/E1.31, RDMnet, and proprietary device APIs.

ALPINE provides:

- **Discovery** — secure device identification without knowing IP
- **Handshake** — mutual authentication & key agreement
- **Control Plane** — reliable, signed envelopes
- **Streaming Layer** — low-latency, real-time lighting frames
- **Capability System** — device declares its features
- **Extensibility** — vendor namespaces, structured envelopes
- **No universes, no DMX limits** — modern frame model

ALPINE is built around:
- **CBOR** for compact structured messages
- **Ed25519** signatures
- **X25519** key exchange
- **UDP broadcast** discovery
- **UDP or QUIC streaming**
- **Deterministic session state machine**

For more details, see the protocol documents:

- [`SPEC.md`](SPEC.md)
- [`docs/architecture.md`](docs/architecture.md)
- [`docs/discovery.md`](docs/discovery.md)
- [`docs/handshake.md`](docs/handshake.md)
- [`docs/control_plane.md`](docs/control_plane.md)
- [`docs/streaming.md`](docs/streaming.md)
- [`docs/capabilities.md`](docs/capabilities.md)
- [`docs/errors.md`](docs/errors.md)
- [`docs/security.md`](docs/security.md)
- [`docs/reference_impl.md`](docs/reference_impl.md)

## Continuous Integration

- `UDP E2E Tests` workflow (`.github/workflows/e2e-tests.yml`) runs `cargo test --tests -- --ignored` from `src/alnp`, exercising the real UDP handshake/control/streaming paths on Linux.

## Publishing & package registries

This project publishes artifacts for Rust, C, TypeScript, and Python. Before running the release scripts or invoking `cargo publish`, set the following credentials in your environment or GitHub secrets:

| Registry | Environment variable | Notes |
| --- | --- | --- |
| GitHub Packages (Rust) | `CARGO_REGISTRIES_GITHUB_TOKEN` | Used by `cargo publish --registry github`. The GitHub registry index is configured in `src/alnp/Cargo.toml`; also point `CARGO_HOME` at `<repo>/.cargo` (or add the same `[registries.github]` entry inside your global Cargo config) so that the registry is loaded before publishing—otherwise `cargo publish --registry github` will panic with “remote registries must have config”. |
| TypeScript (npm/PNPM) | `NPM_TOKEN` or `PNPM_TOKEN` | Required for publishing `dist/ts` via `npm publish` / `pnpm publish`. |
| Python (PyPI or GitHub) | `PYPI_API_TOKEN` (or `TWINE_USERNAME`/`TWINE_PASSWORD`) | `scripts/build_python.sh` generates wheel/sdist artifcats; upload them with `twine upload`. |
| C artifacts | `GITHUB_TOKEN` | Use this token to push `dist/c` (static library + header) to GitHub Packages or release assets. |
| Release validation | — | Follow `docs/release_process.md` before tagging: run `cargo test --manifest-path src/alnp/Cargo.toml`, `scripts/build_c.sh`, `scripts/build_embedded_cpp.sh`, and every binding build so tagging is boring and repeatable. |

## Language Bindings

The reference implementation ships with:

- Rust crate (`alpine-protocol-rs`) exposing `alpine::...`.
- TypeScript client (`@alpine-core/protocol`) built from `bindings/ts`.
- C static library + headers produced by `scripts/build_c.sh`.
- C++ helper header (`bindings/cpp/alnp.hpp`) and embedded-friendly `ALPINE_EMBEDDED` guard.
- Python package (`alpine-protocol`) that mirrors the Rust types.

Each binding provides:

- Discovery
- Handshake
- Session manager
- Control envelope API
- Streaming client/server

## SDK layers

- **Rust**: `src/alnp/src/sdk` exposes `AlpineClient`, which orchestrates discovery, handshake, streaming frames, and keepalive over UDP while reusing the control/crypto helpers.
- **TypeScript**: `bindings/ts/src/sdk/client.ts` wraps the binding helpers with a Node UDP client that exposes `discover()`, `handshake()`, and `sendFrame()` convenience methods.
- **Python**: `bindings/python/src/alnp/sdk` provides a socket-driven class that builds CBOR discovery/control/frame payloads and leaves network I/O to the consumer.
- **C++**: `bindings/cpp/sdk/alpine_sdk.hpp` defines `AlpineTransport` and `AlpineClient` so you can feed encoded discovery/control/frame bytes into your own transport implementation.

These SDKs are the *recommended* application entry points across languages. They orchestrate discovery, handshake, streaming, and keepalive workflows while reusing the underlying binding helpers. Reserve the auto-generated bindings for embedded / constrained environments where the SDK layer cannot run (e.g., ESP32 builds, C-only systems, or highly controlled runtimes).

## Discovery-first SDK workflow

1. Create an `sdk::DiscoveryClient` with your local bind address, broadcast address, requested capabilities, and (optionally) a verifier if you already know the device’s public key.
2. Call `DiscoveryClient::discover().await` to receive a list of deterministic `DiscoveredDevice` entries (addr, `DeviceIdentity`, `CapabilitySet`, and a `signed` flag that notes whether the reply was authenticated).
3. Choose a target device and call `sdk::AlpineClient::connect` with the advertised identity/capabilities.
4. Continue through `start_stream` → `send_frame`, keeping streaming logic contained inside `AlpineClient` and picking `StreamProfile` explicitly.

Discovery now lives entirely outside the session state: the client never auto-connects, never caches results silently, and surfaces every failure so you can explain what happened before choosing a target.

## Stream Profiles

Stream behavior is selected via the `StreamProfile` abstraction exported by the Rust SDK (`StreamProfile::Auto`, `StreamProfile::Realtime`, `StreamProfile::Install`).
Each profile represents a declarative intent (safe default, low latency, or install resilience) and compiles into a stable `config_id`.
Calling `client.start_stream(StreamProfile::Auto)` binds the profile to the session once and never lets the runtime swap it silently; every streaming call thereafter respects the profile weights for latency, resilience, and jitter.
Expect the SDK to reject invalid combinations and to document the behavioral guarantees for every exposed profile so consumers understand what changes under packet loss, jitter, or timing pressure.

## Documentation as API contract

ALPINE treats documentation as part of the API contract. Every public surface across Rust, TypeScript/JavaScript, C, C++, and Python must explain not only "how" but "what the system guarantees" under latency, packet loss, and load. See `docs/documentation_policy.md` for the language-by-language requirements (doc comments, JSDoc, Doxygen, docstrings, deprecation paths, behavioral guarantees, etc.).

## SDK vs. low-level bindings

Every language ships two layers:

1. **High-level SDK** (Rust `src/alnp/src/sdk`, TypeScript `bindings/ts/src/sdk`, Python `bindings/python/src/alnp/sdk`, C++ `bindings/cpp/sdk/alpine_sdk.hpp`): this layer is the recommended entry point for most applications. It hides discovery/handshake plumbing behind idiomatic helpers like `connect()`, `start_stream()`, `send_frame()`, and control/keepalive utilities while enforcing stream profiles and config IDs.
2. **Low-level bindings** (`bindings/c`, `bindings/cpp/alnp.hpp`, `bindings/js`, `bindings/python/src/alnp`): these provide the raw CBOR helpers, enabling strict embedded or allocation-free environments. They intentionally lack runtime conveniences so the SDK keeps doing the heavy lifting for general applications.

Start with the SDKs wherever possible, and fall back to the bindings when you must manage buffers, heap usage, or exotic platforms yourself.

## Embedded mode

The C++ helper exposes an `ALPINE_EMBEDDED` configuration that keeps the API
identical while disabling hidden heap allocations, RTTI, and exception support.
Use `scripts/build_embedded_cpp.sh` or `bindings/cpp/embedded_test.cpp` to verify
the restricted flags (`-fno-exceptions`, `-fno-rtti`, `-fno-threadsafe-statics`,
`-fno-use-cxa-atexit`, `-Os`). The CI job `.github/workflows/embedded.yml` runs
that script for every change to `main`, so ESP32-style builds are validated
alongside the desktop releases.

## Release & CI stability

ALPINE relies on a consistent release flow so that tagging the repository is boring:

1. Run `cargo test --manifest-path src/alnp/Cargo.toml` to verify every SDK helper, profile test, and E2E suite.
2. Run `scripts/build_c.sh` (which runs `cargo build --release`, copies `libalpine.a`, and stages `bindings/c`) and confirm `dist/c` contains the published headers/libraries.
3. Run `scripts/build_embedded_cpp.sh` to prove the constrained C++ build still links against `libalpine-<version>.a` with `ALPINE_EMBEDDED` flags.
4. Run the TypeScript + Python build scripts described in `docs/release_process.md` so the published clients match the SDK guarantees.
5. Tag the repository (e.g., `v1.2.2`), push the tag, and let the release workflows publish the artifacts and packages.

Documenting these steps in `docs/release_process.md` keeps CI green and CI parties comfortable that release builds are repeatable and boring.

## License

Apache-2.0
