#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
ANDROID_DIR="$PROJECT_ROOT/android"
JNI_LIBS_DIR="$ANDROID_DIR/app/src/main/jniLibs"
CARGO_TOML="$PROJECT_ROOT/torvox-gui-android/Cargo.toml"
TARGET_DIR="$PROJECT_ROOT/target"

: "${ANDROID_NDK_ROOT:?ANDROID_NDK_ROOT must be set}"

if ! command -v cargo-ndk &>/dev/null; then
	echo "Installing cargo-ndk..."
	cargo install cargo-ndk
fi

echo "=== Cross-compiling torvox-gui-android for Android ==="

ABIS=("arm64-v8a" "x86_64")

cargo ndk -t arm64-v8a -t x86_64 -o "$TARGET_DIR" build --manifest-path "$CARGO_TOML" --profile dev

for ABI in "${ABIS[@]}"; do
	mkdir -p "$JNI_LIBS_DIR/$ABI"
	cp "$TARGET_DIR/$ABI/libtorvox_core.so" "$JNI_LIBS_DIR/$ABI/"
	echo "Copied to $JNI_LIBS_DIR/$ABI/libtorvox_core.so"
done

echo "=== Done ==="
