#!/bin/sh
set -eu

ROOT_DIR=$(cd "$(dirname "$0")/.." && pwd)
DIST="$ROOT_DIR/dist/embedded"
mkdir -p "$DIST"

cd "$ROOT_DIR"

EMBEDDED_FLAGS="-DALPINE_EMBEDDED -std=c++17 -Wall -Wextra -Werror -fno-exceptions \
-fno-rtti -fno-threadsafe-statics -fno-use-cxa-atexit -Os -ffunction-sections \
-fdata-sections -fno-common -fno-stack-protector"

g++ $EMBEDDED_FLAGS -Ibindings/cpp -Ibindings/c \
  bindings/cpp/embedded_test.cpp -o "$DIST/embedded_test"

echo "Embedded C++ binding built successfully (--embedded mode)."
