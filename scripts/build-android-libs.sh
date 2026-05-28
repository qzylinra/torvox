#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
ANDROID_DIR="$PROJECT_ROOT/android"
JNI_LIBS_DIR="$ANDROID_DIR/app/src/main/jniLibs"
EXEC_DIR="$ANDROID_DIR/app/src/main/assets/bin"
LIB_CARGO_TOML="$PROJECT_ROOT/torvox-gui-android/Cargo.toml"
TARGET_DIR="$PROJECT_ROOT/target"

: "${ANDROID_NDK_ROOT:?ANDROID_NDK_ROOT must be set}"

if ! command -v cargo-ndk &>/dev/null; then
	echo "Installing cargo-ndk..."
	cargo install cargo-ndk
fi

ABIS=("arm64-v8a" "x86_64")
TRIPLES=("aarch64-linux-android" "x86_64-linux-android")
LINKERS=(
	"$ANDROID_NDK_ROOT/toolchains/llvm/prebuilt/linux-x86_64/bin/aarch64-linux-android33-clang"
	"$ANDROID_NDK_ROOT/toolchains/llvm/prebuilt/linux-x86_64/bin/x86_64-linux-android33-clang"
)

echo "=== Cross-compiling torvox-gui-android (cdylib) ==="
cargo ndk -t arm64-v8a -t x86_64 -o "$TARGET_DIR" build --manifest-path "$LIB_CARGO_TOML" --profile dev

for ABI in "${ABIS[@]}"; do
	mkdir -p "$JNI_LIBS_DIR/$ABI"
	cp "$TARGET_DIR/$ABI/libtorvox_core.so" "$JNI_LIBS_DIR/$ABI/"
	echo "Copied to $JNI_LIBS_DIR/$ABI/libtorvox_core.so"
done

echo "=== Cross-compiling torvox-exec (PIE binary) ==="
for i in "${!ABIS[@]}"; do
	ABI="${ABIS[$i]}"
	TRIPLE="${TRIPLES[$i]}"
	LINKER="${LINKERS[$i]}"
	echo "--- Building torvox-exec for $TRIPLE ($ABI) ---"
	ENV_VAR="CARGO_TARGET_$(echo "$TRIPLE" | tr 'a-z-' 'A-Z_')_LINKER"
	export "$ENV_VAR"="$LINKER"
	cargo build -p torvox-exec --target "$TRIPLE" --profile dev
	unset "$ENV_VAR"
	mkdir -p "$EXEC_DIR/$ABI"
	cp "$TARGET_DIR/$TRIPLE/debug/torvox-exec" "$EXEC_DIR/$ABI/"
	chmod +x "$EXEC_DIR/$ABI/torvox-exec"
	echo "Copied to $EXEC_DIR/$ABI/torvox-exec"
done

echo "=== Done ==="
