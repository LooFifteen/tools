{
  description = "A very basic flake";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { nixpkgs, rust-overlay, ... }:
    let
      system = "x86_64-linux";
      pkgs = import nixpkgs {
        inherit system;
        overlays = [ (import rust-overlay) ];
      };
      rustToolchain = pkgs.pkgsBuildHost.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
      nativeBuildInputs = with pkgs; [
        pkg-config
        rustToolchain
      ];
      buildInputs = with pkgs; [ openssl ];
    in
    with pkgs;
    {
      devShells.${system}.default = mkShell {
        inherit nativeBuildInputs buildInputs;
      };
    };
}
