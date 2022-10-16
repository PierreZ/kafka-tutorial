{
  description = "A dev environment for the crate foundationdb-rs";
  inputs = {
    nixpkgs.url = "nixpkgs";
    rust-overlay.url = "github:oxalica/rust-overlay";
    rust-overlay.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs = all@{ self, nixpkgs,  rust-overlay, ... }: {
    # Utilized by `nix develop`
    devShells.x86_64-linux.default =
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit overlays;
          system = "x86_64-linux";
        };
      in
      with pkgs;
      mkShell {
        buildInputs = [
          # bindgen part
          clang
          llvmPackages.libclang
          llvmPackages.libcxxClang
          pkg-config

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
  };
}