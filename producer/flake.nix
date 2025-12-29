{
  description = "A dev environment for the Kafka producer";
  inputs = {
    nixpkgs.url = "nixpkgs";
    rust-overlay.url = "github:oxalica/rust-overlay";
    rust-overlay.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs = { self, nixpkgs, rust-overlay, ... }:
    let
      supportedSystems = [ "x86_64-linux" "aarch64-linux" "x86_64-darwin" "aarch64-darwin" ];
      forAllSystems = nixpkgs.lib.genAttrs supportedSystems;
    in
    {
      devShells = forAllSystems (system:
        let
          overlays = [ (import rust-overlay) ];
          pkgs = import nixpkgs {
            inherit system overlays;
          };
        in
        {
          default = with pkgs; mkShell {
            buildInputs = [
              # bindgen part
              clang
              llvmPackages.libclang
              llvmPackages.libcxxClang
              pkg-config
              openssl
              cmake

              # Rust part
              cargo-expand
              cargo-edit
              rust-analyzer
              (rust-bin.stable.latest.default.override {
                extensions = [ "rust-src" ];
              })
              rust-bin.stable.latest.rustfmt
              rust-bin.stable.latest.clippy
            ];

            # https://github.com/NixOS/nixpkgs/issues/52447#issuecomment-853429315
            BINDGEN_EXTRA_CLANG_ARGS = "-isystem ${llvmPackages.libclang.lib}/lib/clang/${lib.getVersion clang}/include";
            LIBCLANG_PATH = "${llvmPackages.libclang.lib}/lib";
          };
        });
    };
}