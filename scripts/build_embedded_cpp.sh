#!/bin/sh
set -euo pipefail
set -x

ROOT_DIR=$(cd "$(dirname "$0")/.." && pwd)
DIST="$ROOT_DIR/dist/embedded"
mkdir -p "$DIST"

cd "$ROOT_DIR"
./scripts/build_c.sh

EMBEDDED_FLAGS="-DALPINE_EMBEDDED -std=c++17 -Wall -Wextra -Werror -fno-exceptions \
-fno-rtti -fno-threadsafe-statics -fno-use-cxa-atexit -Os -ffunction-sections \
-fdata-sections -fno-common -fno-stack-protector"

VERSION=$(cat "$ROOT_DIR/VERSION")

ls -l "$ROOT_DIR/dist/c"

g++ $EMBEDDED_FLAGS -Iprotocol/cpp -Iprotocol/c \
  protocol/cpp/embedded_test.cpp -L"$ROOT_DIR/dist/c" "-l:libalpine-$VERSION.a" \
  -o "$DIST/embedded_test"

echo "Embedded C++ binding built successfully (--embedded mode)."
