{
  description = "Torvox — Android terminal emulator (Rust engine + Kotlin UI)";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-parts.url = "github:hercules-ci/flake-parts";
  };

  outputs =
    inputs@{
      self,
      nixpkgs,
      fenix,
      flake-parts,
      ...
    }:
    flake-parts.lib.mkFlake { inherit inputs; } {
      systems = [
        "x86_64-linux"
        "aarch64-linux"
        "x86_64-darwin"
        "aarch64-darwin"
      ];

      perSystem =
        {
          pkgs,
          system,
          ...
        }:
        let
          pkgs' = import nixpkgs {
            inherit system;
            config.allowUnfree = true;
          };

          fenixPkgs = fenix.packages.${system};

          rustToolchain = fenixPkgs.stable.withComponents [
            "cargo"
            "clippy"
            "rust-src"
            "rust-std"
            "rustc"
            "rustfmt"
          ];

          androidSdkPkgs = pkgs'.androidenv.composeAndroidPackages.override { licenseAccepted = true; } {
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

          jdk = pkgs'.javaPackages.compiler.temurin-bin.jdk-25;

          nativeDeps = with pkgs; [
            pkg-config
            openssl
          ];

          rustDeps = [
            rustToolchain
            pkgs.cargo-nextest
            pkgs.cargo-fuzz
            pkgs.cargo-geiger
            pkgs.cargo-audit
            pkgs.rust-analyzer
          ];

          androidDeps = with pkgs'; [
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
          devShells.default = pkgs.mkShell {
            name = "torvox-dev";

            packages = nativeDeps ++ rustDeps ++ androidDeps;

            LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath nativeDeps;

            RUST_SRC_PATH = "${rustToolchain}/lib/rustlib/src/rust/library";

            ANDROID_HOME = "${androidSdkPkgs.androidsdk}/libexec/android-sdk";
            ANDROID_SDK_ROOT = "${androidSdkPkgs.androidsdk}/libexec/android-sdk";
            ANDROID_NDK_ROOT = "${androidSdkPkgs.androidsdk}/libexec/android-sdk/ndk/29.0.14206865";

            JAVA_HOME = jdk;

            shellHook = ''
              echo "=== Torvox Dev Shell ==="
              echo "Rust:     $(rustc --version)"
              echo "Cargo:    $(cargo --version)"
              echo "Nextest:  $(cargo nextest --version 2>/dev/null || echo N/A)"
              echo "Kotlin:   $(kotlin -version 2>&1 | head -1 || echo N/A)"
              echo "Gradle:   $(gradle --version 2>/dev/null | grep '^Gradle' || echo N/A)"
              echo "JDK:      $(java -version 2>&1 | head -1)"
              echo "ANDROID_HOME: $ANDROID_HOME"
              echo ""
              echo "Quick start:"
              echo "  cargo build --workspace"
              echo "  cargo nextest run --workspace"
              echo "  cargo clippy -- -D warnings"
              echo "  cd android && ./gradlew assembleDebug"
            '';
          };
        };
    };
}
