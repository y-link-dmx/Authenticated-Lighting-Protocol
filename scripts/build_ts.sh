#!/bin/sh
set -e

ROOT_DIR=$(cd "$(dirname "$0")/.." && pwd)
VERSION=$(cat "$ROOT_DIR/VERSION")
DIST="$ROOT_DIR/dist/ts"

mkdir -p "$DIST"
cd "$ROOT_DIR/protocol/ts"

if command -v pnpm >/dev/null 2>&1; then
  PM=pnpm
else
  PM=npm
fi

echo "==> Installing dependencies ($PM)"
$PM install
echo "==> Building TypeScript package (version $VERSION)"
$PM run build

cp -r dist "$DIST/"
cp package.json "$DIST/package.json"
echo "TypeScript artifacts written to $DIST"
