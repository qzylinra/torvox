#!/bin/bash
# Generate Kotlin bindings with boltffi
# Prerequisites: cargo-ndk installed, ANDROID_NDK_ROOT set
# Usage: ./scripts/generate-bindings.sh

set -e

echo "=== Building torvox-gui-android for Android ==="
cargo ndk -t arm64-v8a -t x86_64 \
	--manifest-path torvox-gui-android/Cargo.toml \
	-o target/ndk \
	build --release

echo "=== Generating Kotlin bindings ==="
boltffi generate kotlin \
	--output android/app/src/main/java/io/torvox/bridge/

echo "=== Done ==="
echo "Generated files in android/app/src/main/java/io/torvox/bridge/"
