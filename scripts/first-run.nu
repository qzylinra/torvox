#!/usr/bin/env nu
# First-run setup for Torvox (nushell)
# All tools come from nix develop — no cargo install or manual toolchain setup.

if (which nix | length) > 0 and ("NIX_DEVELOP_ENV" not-in $env) {
    exec nix develop --command nu $env.CURRENT_FILE
}

# ── Clone libghostty-rs ───────────────────────────────────────────
print "=== Cloning libghostty-rs ==="
if not ("libghostty-rs" | path exists) {
    ^git clone --depth 1 https://github.com/Uzaaft/libghostty-rs.git libghostty-rs
} else {
    print "libghostty-rs already present"
}

# ── Build native Rust library for Android (x86_64) ────────────────
print "=== Building native Rust library for Android (x86_64) ==="
$env.ANDROID_NDK_HOME = ($env.ANDROID_NDK_HOME? | default "/usr/local/lib/android/sdk/ndk/27.3.13750724")
^cargo ndk -t x86_64 -P 27 build --release -p torvox-gui-android

# ── Copy .so to jniLibs ───────────────────────────────────────────
print "=== Copying .so to jniLibs ==="
mkdir android/app/src/main/jniLibs/x86_64
^cp target/x86_64-linux-android/release/libtorvox_android.so android/app/src/main/jniLibs/x86_64/

# ── Build APK ─────────────────────────────────────────────────────
print "=== Building APK ==="
cd android
^./gradlew assembleDebug
cd ..

print "=== Done ==="
print "APK: android/app/build/outputs/apk/debug/app-debug.apk"
