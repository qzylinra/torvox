{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-parts = {
      url = "github:hercules-ci/flake-parts";
      inputs.nixpkgs-lib.follows = "nixpkgs";
    };
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      nixpkgs,
      fenix,
      flake-parts,
      ...
    }@inputs:
    flake-parts.lib.mkFlake { inherit inputs; } {
      systems = builtins.filter (system: system != "x86_64-darwin") nixpkgs.lib.systems.flakeExposed;

      perSystem =
        {
          pkgs,
          system,
          ...
        }:
        let
          rustToolchain = pkgs.fenix.stable.withComponents [
            "cargo"
            "clippy"
            "rust-src"
            "rustc"
            "rustfmt"
          ];
          rustToolchainAndroid = pkgs.fenix.combine [
            pkgs.fenix.targets
            pkgs.fenix.targets.aarch64-linux-android.stable.rust-std
            pkgs.fenix.targets.x86_64-linux-android.stable.rust-std
          ];
          androidSdkPkgs = pkgs.androidenv.composeAndroidPackages.override { licenseAccepted = true; } {
            buildToolsVersions = [ "36.0.0" ];
            cmdLineToolsVersion = "16.0";
            platformToolsVersion = "36.0.0";
            platformVersions = [
              "36"
              "33"
            ];
            ndkVersions = [ "29.0.14206865" ];
            includeNDK = true;
            includeEmulator = false;
            includeSystemImages = false;
            includeSources = false;
          };
          androidEmuSdkPkgs = pkgs.androidenv.composeAndroidPackages.override { licenseAccepted = true; } {
            buildToolsVersions = [ "36.0.0" ];
            cmdLineToolsVersion = "16.0";
            platformToolsVersion = "36.0.0";
            platformVersions = [
              "36"
              "33"
            ];
            ndkVersions = [ "29.0.14206865" ];
            includeNDK = true;
            includeEmulator = true;
            includeSystemImages = true;
            systemImageTypes = [ "default" ];
            abiVersions = [ "x86_64" ];
            includeSources = false;
          };
          jdk = pkgs.javaPackages.compiler.temurin-bin.jdk-25;
          nativeDeps = with pkgs; [
            pkg-config
            openssl
          ];
          rustDeps = [
            rustToolchainAndroid
            pkgs.cargo-nextest
            pkgs.cargo-fuzz
            pkgs.cargo-geiger
            pkgs.cargo-audit
            pkgs.cargo-ndk
            pkgs.rust-analyzer
          ];
          androidDeps = with pkgs; [
            jdk
            kotlin
            gradle_9
            ktfmt
            ktlint
            android-tools
            androidSdkPkgs.androidsdk
          ];
        in
        {
          _module.args.pkgs = import nixpkgs {
            inherit system;
            config = {
              allowUnfree = true;
              allowAliases = false;
              warnUndeclaredOptions = true;
            };
            overlays = [
              fenix.overlays.default
            ];
          };

          devShells = {
            default = pkgs.mkShell {
              name = "torvox-dev";
              packages = nativeDeps ++ rustDeps ++ androidDeps;
              env = {
                LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath nativeDeps;
                RUST_SRC_PATH = "${rustToolchain}/lib/rustlib/src/rust/library";
                ANDROID_HOME = "${androidSdkPkgs.androidsdk}/libexec/android-sdk";
                ANDROID_SDK_ROOT = "${androidSdkPkgs.androidsdk}/libexec/android-sdk";
                ANDROID_NDK_ROOT = "${androidSdkPkgs.androidsdk}/libexec/android-sdk/ndk/29.0.14206865";
                JAVA_HOME = jdk;
              };
              shellHook = ''
                echo "=== Torvox Dev Shell ==="
                echo "Rust: $(rustc --version)"
                echo "Cargo: $(cargo --version)"
                echo "Nextest: $(cargo nextest --version 2>/dev/null || echo N/A)"
                echo "Kotlin: $(kotlin -version 2>&1 | head -1 || echo N/A)"
                echo "Gradle: $(gradle --version 2>/dev/null | grep '^Gradle' || echo N/A)"
                echo "JDK: $(java -version 2>&1 | head -1)"
                echo "ANDROID_HOME: $ANDROID_HOME"
                echo ""
                echo "Quick start:"
                echo " cargo build --workspace"
                echo " cargo nextest run --workspace"
                echo " cargo clippy -- -D warnings"
                echo " cd android && ./gradlew assembleDebug"
              '';
            };
            emulator = pkgs.mkShell {
              name = "torvox-emulator";
              packages =
                nativeDeps
                ++ rustDeps
                ++ androidDeps
                ++ [
                  androidEmuSdkPkgs.androidsdk
                  pkgs.qemu
                ];
              env = {
                LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath nativeDeps;
                RUST_SRC_PATH = "${rustToolchain}/lib/rustlib/src/rust/library";
                ANDROID_HOME = "${androidEmuSdkPkgs.androidsdk}/libexec/android-sdk";
                ANDROID_SDK_ROOT = "${androidEmuSdkPkgs.androidsdk}/libexec/android-sdk";
                ANDROID_NDK_ROOT = "${androidEmuSdkPkgs.androidsdk}/libexec/android-sdk/ndk/29.0.14206865";
                JAVA_HOME = jdk;
              };
              shellHook = ''
                echo "=== Torvox Emulator Shell ==="
                echo "Emulator SDK: $ANDROID_HOME"
                echo "Run: emulator -avd torvox_api36 -no-window -no-boot-anim -noaudio"
                echo "Setup AVD: avdmanager create avd -n torvox_api36 -k 'system-images;android-36;default;x86_64' -d pixel_7_pro"
              '';
            };
          };
        };
    };
}
