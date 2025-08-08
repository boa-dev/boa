{
  description = "Boa engine dev shell";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      self,
      nixpkgs,
      rust-overlay,
      flake-utils,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ rust-overlay.overlays.default ];
        };
        rust-toolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [
            "rust-src"
            "rust-analyzer"
          ];
        };
      in
      {
        devShells.default =
          with pkgs;
          mkShell rec {
            packages = [
              vulkan-tools
            ];
            buildInputs = [
              openssl
              vulkan-loader
            ];
            nativeBuildInputs = [
              rust-toolchain
              pkg-config
            ];
            # Required for jemalloc, see https://github.com/NixOS/nixpkgs/issues/370494 .
            CFLAGS = "-DJEMALLOC_STRERROR_R_RETURNS_CHAR_WITH_GNU_SOURCE";
            LD_LIBRARY_PATH = "${lib.makeLibraryPath buildInputs}";
          };
      }
    );
}
