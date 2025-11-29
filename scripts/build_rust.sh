#!/bin/sh
set -e

ROOT_DIR=$(cd "$(dirname "$0")/.." && pwd)
VERSION=$(cat "$ROOT_DIR/VERSION")
DIST="$ROOT_DIR/dist/rust"

mkdir -p "$DIST"
cd "$ROOT_DIR/protocol/rust/alpine-protocol-rs"

echo "==> Building Rust crate (version $VERSION)"
cargo test
echo "==> Running UDP E2E tests (cargo test --tests -- --ignored)"
cargo test --tests -- --ignored
cargo build --release

echo "==> Packaging artifacts"
cp -f target/release/libalnp.a "$DIST/libalnp-$VERSION.a" 2>/dev/null || true
cargo package --allow-dirty --no-verify
cp -f target/package/alnp-$VERSION.crate "$DIST/alnp-$VERSION.crate"

echo "Rust artifacts written to $DIST"
