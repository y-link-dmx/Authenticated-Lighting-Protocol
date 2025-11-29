#!/bin/sh
set -e

ROOT_DIR=$(cd "$(dirname "$0")/.." && pwd)
VERSION=$(cat "$ROOT_DIR/VERSION")
DIST="$ROOT_DIR/dist/python"

mkdir -p "$DIST"
cd "$ROOT_DIR/protocol/python"

echo "==> Building Python package (version $VERSION)"
python -m pip install -U build >/dev/null
python -m build

cp -r dist "$DIST/"
echo "Python artifacts written to $DIST"
