{
  description = "A very basic flake";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    crane.url = "github:ipetkov/crane";
  };

  outputs = { nixpkgs, rust-overlay, crane, ... }:
    let
      system = "x86_64-linux";
      pkgs = import nixpkgs {
        inherit system;
        overlays = [ (import rust-overlay) ];
      };
      rustToolchain = pkgs.pkgsBuildHost.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;

      # crane
      craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;
      templateFilter = path: type: type == "regular" && builtins.match ".*/src/templates/.*" path != null;
      templateOrCargo = path: type: (templateFilter path type) || (craneLib.filterCargoSources path type);
      src = pkgs.lib.cleanSourceWith {
        src = ./.;
        filter = templateOrCargo;
        name = "source";
      };

      # build inputs
      nativeBuildInputs = with pkgs; [
        pkg-config
        rustToolchain
      ];
      buildInputs = with pkgs; [ openssl ];

      # arguments
      commonArguments = {
        inherit src buildInputs nativeBuildInputs;
      };
      cargoArtifacts = craneLib.buildDepsOnly commonArguments;

      # binary
      bin = craneLib.buildPackage (commonArguments // {
        inherit cargoArtifacts;
      });

      # docker
      dockerImage = pkgs.dockerTools.streamLayeredImage {
        name = "tools";
        tag = "latest";
        contents = [ bin ];
        config = {
          Cmd = [ "${bin}/bin/tools" ];
          ExposedPorts = [ "3000/tcp" ];
        };
      };
    in
    with pkgs;
    {
      packages.${system} =
        {
          inherit bin dockerImage;
          default = bin;
        };
      devShells.${system}.default = mkShell {
        inputsFrom = [ bin ];
      };
    };
}
