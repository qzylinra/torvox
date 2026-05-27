{
  description = "Torvox — Android terminal emulator (Rust engine + Kotlin UI)";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      self,
      nixpkgs,
      fenix,
      flake-utils,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
        fenixPkgs = fenix.packages.${system};

        rustToolchain = fenixPkgs.stable.withComponents [
          "cargo"
          "clippy"
          "rust-src"
          "rust-std"
          "rustc"
          "rustfmt"
        ];

        rustTargetPkgs = with pkgs; [
          pkg-config
          openssl
        ];

        devTools = with pkgs; [
          rustToolchain
          cargo-nextest
          cargo-fuzz
          cargo-geiger
          cargo-audit
          rust-analyzer
        ];
      in
      {
        devShells.default = pkgs.mkShell {
          name = "torvox-dev";

          packages = devTools ++ rustTargetPkgs;

          LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath rustTargetPkgs;

          RUST_SRC_PATH = "${rustToolchain}/lib/rustlib/src/rust/library";

          shellHook = ''
            echo "=== Torvox Dev Shell ==="
            echo "rustc: $(rustc --version)"
            echo "cargo: $(cargo --version)"
            echo "cargo-nextest: $(cargo nextest --version 2>/dev/null || echo 'N/A')"
            echo ""
            echo "Quick start:"
            echo "  cargo build --workspace"
            echo "  cargo nextest run --workspace"
            echo "  cargo clippy --deny warnings"
          '';
        };
      }
    );
}
