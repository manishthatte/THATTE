{
  description = "THATTE extended starter";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-24.05";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ rust-overlay.overlays.default ];
        pkgs = import nixpkgs { inherit system overlays; };
        rustToolchain = pkgs.rust-bin.nightly.latest.default.override {
          extensions = [ "rust-src" "rustfmt" ];
          targets = [ "x86_64-unknown-uefi" "x86_64-unknown-linux-musl" ];
        };
      in {
        devShells.default = pkgs.mkShell {
          packages = [
            rustToolchain
            pkgs.qemu pkgs.OVMF
            pkgs.mtools pkgs.dosfstools
            pkgs.llvm pkgs.clang pkgs.lld
            pkgs.gnumake
          ];
        };
      });
}
