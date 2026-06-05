{
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
            config.allowUnfree = true;
            overlays = [ inputs.fenix.overlays.default ];
          };
          packages.rust-toolchain = pkgs.fenix.stable.withComponents [
            "cargo"
            "clippy"
            "rust-src"
            "rustc"
            "rustfmt"
          ];
          packages.nightly-toolchain = pkgs.fenix.latest.withComponents [
            "cargo"
            "clippy"
            "miri"
            "rust-src"
            "rustc"
            "rustfmt"
          ];
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
              (fenix.stable.withComponents [
                "cargo"
                "clippy"
                "rust-src"
                "rustc"
                "rustfmt"
              ])
              (fenix.latest.withComponents [
                "cargo"
                "clippy"
                "miri"
                "rust-src"
                "rustc"
                "rustfmt"
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
              openspec
              pkg-config
              openssl
              zig_0_15
              git
            ];
            env.LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath (
              with pkgs;
              [
                pkg-config
                openssl
              ]
            );
          };
        };
    };
}
