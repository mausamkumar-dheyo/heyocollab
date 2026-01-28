{
  description = "heyocollab - Rust CRDT library with WASM bindings";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };

        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-src" "rust-analyzer" ];
          targets = [ "wasm32-unknown-unknown" ];
        };
      in
      {
        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            # Rust toolchain with WASM target
            rustToolchain

            # WASM tools
            wasm-pack
            wasm-bindgen-cli
            binaryen  # wasm-opt

            # Build dependencies
            pkg-config
            openssl

            # Node.js for testing WASM output
            nodejs_20
            nodePackages.npm
          ];

          shellHook = ''
            echo "heyocollab development environment"
            echo "Rust: $(rustc --version)"
            echo "wasm-pack: $(wasm-pack --version)"
            echo ""
            echo "Build WASM with: wasm-pack build --target web --out-dir pkg --features wasm --release"
          '';
        };
      }
    );
}
