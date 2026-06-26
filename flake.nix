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
            config.allowUnfree = true;
            overlays = [ inputs.fenix.overlays.default ];
          };
          packages = {
            rust-toolchain = pkgs.fenix.stable.withComponents [
              "cargo"
              "clippy"
              "rust-src"
              "rustc"
              "rustfmt"
            ];
            nightly-toolchain = pkgs.fenix.latest.withComponents [
              "cargo"
              "clippy"
              "miri"
              "rust-src"
              "rustc"
              "rustfmt"
            ];
          };
          formatter = pkgs.nixfmt-tree.override {
            nixfmtPackage = pkgs.nixfmt-rs;
            runtimeInputs = with pkgs; [
              taplo
              yamlfmt
              rustfmt
              typos
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
              rustfmt = {
                command = "rustfmt";
                options = [
                  "--config"
                  "skip_children=true"
                  "--edition"
                  "2024"
                  "--style-edition"
                  "2024"
                ];
                includes = [ "*.rs" ];
              };
              typos = {
                command = "typos";
                includes = [
                  "*.rs"
                  "*.kt"
                  "*.md"
                  "*.toml"
                  "*.nix"
                  "*.yml"
                  "*.yaml"
                ];
              };
            };
          };
          devShells.default = pkgs.mkShell {
            name = "torvox-dev";
            packages = with pkgs; [
              (fenix.combine [
                (fenix.latest.withComponents [
                  "cargo"
                  "clippy"
                  "miri"
                  "rust-src"
                  "rustc"
                  "rustfmt"
                ])
                fenix.targets.thumbv6m-none-eabi.latest.rust-std
                fenix.targets.x86_64-linux-android.latest.rust-std
                fenix.targets.aarch64-linux-android.latest.rust-std
              ])
              (fenix.combine [
                (fenix.stable.withComponents [
                  "cargo"
                  "clippy"
                  "rust-src"
                  "rustc"
                  "rustfmt"
                ])
                fenix.targets.thumbv6m-none-eabi.stable.rust-std
              ])
              cargo-nextest
              cargo-fuzz
              cargo-geiger
              cargo-audit
              cargo-ndk
              cargo-deny
              cargo-machete
              cargo-mutants
              rust-analyzer
              kotlin
              gradle_9
              jdk
              ktfmt
              ktlint
              android-tools
              nushell
              taplo
              yamlfmt
              typos
              markdownlint-cli2
              mesa
              vulkan-loader
              vulkan-tools
              nixfmt-rs
              statix
              deadnix
              pkg-config
              openssl
              zig_0_15
              maestro
              stdenv.cc.cc.lib
              (python3.withPackages (
                ps: with ps; [
                  pyte
                  sphinx
                  sphinx-rtd-theme
                  pip
                ]
              ))
              git
              curl
              jq
              gnutar
              gzip
              patch
            ];
            env.LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath (
              with pkgs;
              [
                pkg-config
                openssl
                vulkan-loader
                mesa
                gcc.cc.lib
              ]
            );
            shellHook = ''
              export GHOSTTY_SOURCE_DIR="$(nu scripts/bootstrap-libghostty.nu | tail -1)"
              export VK_ICD_FILENAMES="${pkgs.mesa}/share/vulkan/icd.d/lvp_icd.x86_64.json"
            '';
          };
        };
    };
}
