#!/bin/sh
# Minimal install: build and place the binary in ~/.local/bin.
set -e

SRC_DIR=$(cd "$(dirname "$0")" && pwd)
BIN="openclaude-zen-free"
DEST_DIR="${HOME}/.local/bin"
DEST="${DEST_DIR}/${BIN}"

command -v go >/dev/null 2>&1 || { echo "go is required (https://go.dev/dl)"; exit 1; }

echo "Building ${BIN} ..."
( cd "$SRC_DIR" && CGO_ENABLED=0 go build -o "$BIN" . )

mkdir -p "$DEST_DIR"
install -m 0755 "${SRC_DIR}/${BIN}" "$DEST"
echo "Installed: $DEST"
echo "Make sure $DEST_DIR is on your PATH, then run: $BIN"
