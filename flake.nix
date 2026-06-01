{
  description = "Torvox — Android 终端模拟器";
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";
    fenix.url = "github:nix-community/fenix";
  };
  outputs =
    inputs:
    inputs.flake-parts.lib.mkFlake { inherit inputs; } {
      systems = [
        "x86_64-linux"
        "aarch64-linux"
      ];
      perSystem =
        {
          pkgs,
          system,
          ...
        }:
        {
          _module.args.pkgs = import inputs.nixpkgs {
            inherit system;
            config.allowUnfreePredicate =
              package:
              builtins.elem package.pname [
                "androidsdk"
                "android-studio-stable"
                "jdk"
              ];
            overlays = [ inputs.fenix.overlays.default ];
          };
          packages.rust-toolchain = pkgs.fenix.stable.withComponents [
            "cargo"
            "clippy"
            "rust-src"
            "rustc"
            "rustfmt"
          ];
          formatter = pkgs.nixfmt-tree.override {
            nixfmtPackage = pkgs.nixfmt;
            runtimeInputs = [
              pkgs.taplo
              pkgs.yamlfmt
            ];
            settings.formatter = {
              toml = {
                command = "taplo";
                options = [ "format" ];
                includes = [ "*.toml" ];
              };
              yaml = {
                command = "yamlfmt";
                includes = [
                  "*.yaml"
                  "*.yml"
                ];
              };
            };
          };
          checks =
            let
              toolchain = pkgs.fenix.stable.withComponents [
                "cargo"
                "clippy"
                "rust-src"
                "rustc"
                "rustfmt"
              ];
              nativeDependencies = [
                toolchain
                pkgs.cargo-nextest
                pkgs.pkg-config
                pkgs.openssl
              ];
              libraryPath = pkgs.lib.makeLibraryPath [
                pkgs.pkg-config
                pkgs.openssl
              ];
              copy_source = "cp -r ${./.} . && chmod -R u+w .";
              body = {
                clippy =
                  pkgs.runCommand "check-clippy"
                    {
                      nativeBuildInputs = nativeDependencies;
                      LD_LIBRARY_PATH = libraryPath;
                    }
                    ''
                      ${copy_source}
                      cargo clippy -- -D warnings
                      touch $out
                    '';
                fmt =
                  pkgs.runCommand "check-fmt"
                    {
                      nativeBuildInputs = [ toolchain ];
                    }
                    ''
                      ${copy_source}
                      cargo fmt --check
                      touch $out
                    '';
                tests =
                  pkgs.runCommand "check-tests"
                    {
                      nativeBuildInputs = nativeDependencies;
                      LD_LIBRARY_PATH = libraryPath;
                    }
                    ''
                      ${copy_source}
                      cargo nextest run --workspace
                      touch $out
                    '';
                proptest =
                  pkgs.runCommand "check-proptest"
                    {
                      nativeBuildInputs = nativeDependencies;
                      LD_LIBRARY_PATH = libraryPath;
                    }
                    ''
                      ${copy_source}
                      cargo test --workspace -- proptest
                      touch $out
                    '';
                typos =
                  pkgs.runCommand "check-typos"
                    {
                      nativeBuildInputs = [ pkgs.typos ];
                    }
                    ''
                      ${copy_source}
                      typos
                      touch $out
                    '';
                nixfmt =
                  pkgs.runCommand "check-nixfmt"
                    {
                      nativeBuildInputs = [ pkgs.nixfmt ];
                    }
                    ''
                      ${copy_source}
                      find . -name '*.nix' \
                        -not -path './target/*' \
                        -not -path './.git/*' \
                        -exec nixfmt --check {} +
                      touch $out
                    '';
                cargo-deny =
                  pkgs.runCommand "check-cargo-deny"
                    {
                      nativeBuildInputs = [
                        toolchain
                        pkgs.cargo-deny
                      ];
                    }
                    ''
                      ${copy_source}
                      cargo deny check
                      touch $out
                    '';
                cargo-audit =
                  pkgs.runCommand "check-cargo-audit"
                    {
                      nativeBuildInputs = [
                        toolchain
                        pkgs.cargo-audit
                      ];
                    }
                    ''
                      ${copy_source}
                      cargo audit
                      touch $out
                    '';
                cargo-machete =
                  pkgs.runCommand "check-cargo-machete"
                    {
                      nativeBuildInputs = [
                        toolchain
                        pkgs.cargo-machete
                      ];
                    }
                    ''
                      ${copy_source}
                      cargo machete
                      touch $out
                    '';
                markdownlint =
                  pkgs.runCommand "check-markdownlint"
                    {
                      nativeBuildInputs = [ pkgs.markdownlint-cli2 ];
                    }
                    ''
                      ${copy_source}
                      find . -name '*.md' \
                        -not -path './target/*' \
                        -not -path './.git/*' \
                        -not -path './.opencode/*' \
                        -exec markdownlint-cli2 {} +
                      touch $out
                    '';
              };
            in
            body;
          devShells.default = pkgs.mkShell {
            name = "torvox-dev";
            packages = [
              (pkgs.fenix.stable.withComponents [
                "cargo"
                "clippy"
                "rust-src"
                "rustc"
                "rustfmt"
              ])
              pkgs.cargo-nextest
              pkgs.cargo-fuzz
              pkgs.cargo-geiger
              pkgs.cargo-audit
              pkgs.cargo-ndk
              pkgs.cargo-deny
              pkgs.cargo-machete
              pkgs.rust-analyzer
              pkgs.kotlin
              pkgs.gradle_9
              pkgs.ktfmt
              pkgs.ktlint
              pkgs.android-tools
              pkgs.nushell
              pkgs.taplo
              pkgs.yamlfmt
              pkgs.typos
              pkgs.markdownlint-cli2
              pkgs.pkg-config
              pkgs.openssl
              pkgs.zig_0_15
              pkgs.git
            ];
            env = {
              LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath [
                pkgs.pkg-config
                pkgs.openssl
              ];
            };
          };
        };
    };
}
