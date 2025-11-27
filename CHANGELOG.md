# Changelog

- All notable changes to ALPINE will be documented in this file.

## [1.0.7] - 2025-12-01
- Keep the crate name `alpine-core` for crates.io while exposing the library as `alpine` so existing tests and consumers continue to import `alpine::…`.
- Restore `libalpine.a` as the C artifact while keeping the GHCR image packaging and docs bundle unchanged.
- Continue shipping docs + artifacts together so release pages always include README/SPEC/docs and the tarball on GHCR.

## [1.0.6] - 2025-11-30
- Rename the Rust crate to `alpine-core` and ship it as `alpine-core-1.0.6` so future updates can belong to your organization.
- Update the static library export to `libalpine-core-*.a` so the C bindings still match the crate name.
- Keep docs, GHCR C package, and release notes flowing with the new tag.

## [1.0.5] - 2025-11-29
- Publish the C tarball as both a release asset and a GHCR package so it’s easy to download/play with.
- Bundle README, SPEC, and `docs/` into each release asset so the documentation always travels with every package.
- Keep TS, Python, and Rust manifests in sync with the `v1.0.5` tag so CI redeploys everything cleanly.

## [1.0.4] - 2025-11-29
- Target `crates-io` explicitly when publishing Rust so Cargo knows which registry to hit.
- Refresh every binding manifest/tag to `1.0.4` so the next release has new artifacts.
- Confirm C/TS/Python jobs still upload via GitHub Packages with the new permissions.

## [1.0.2] - 2025-11-27
- Align TS/GitHub package workflows with the `@alpine-core` scope and add npmjs/public flags.
- Fix release artifacts to copy the actual crate/static lib names and expose Python wheels.
- Tag the repo `v1.0.2` so CI can publish all bindings again.

## [1.0.0] - 2025-11-23
- First public release of ALPINE v1.
- Deterministic session state machine and authenticated control plane over UDP.
- X25519 + Ed25519 security model with signed control envelopes.
- Reliable control channel (retransmit/backoff/replay protection).
- ALNP-Stream gating with jitter handling (hold-last, drop, lerp).
- TypeScript and C bindings scaffolds; Python package stub for clients.

[1.0.7]: https://github.com/alpine-core/Authenticated-Lighting-Protocol/releases/tag/v1.0.7
[1.0.6]: https://github.com/alpine-core/Authenticated-Lighting-Protocol/releases/tag/v1.0.6
[1.0.5]: https://github.com/alpine-core/Authenticated-Lighting-Protocol/releases/tag/v1.0.5
[1.0.4]: https://github.com/alpine-core/Authenticated-Lighting-Protocol/releases/tag/v1.0.4
[1.0.3]: https://github.com/alpine-core/Authenticated-Lighting-Protocol/releases/tag/v1.0.3
[1.0.2]: https://github.com/alpine-core/Authenticated-Lighting-Protocol/releases/tag/v1.0.2
[1.0.0]: https://github.com/alpine-core/Authenticated-Lighting-Protocol/releases/tag/v1.0.0
