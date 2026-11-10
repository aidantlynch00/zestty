{
  description = "";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    crane.url = "github:ipetkov/crane";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, crane, flake-utils, rust-overlay, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ (import rust-overlay) ];
        };

        rustWithWasiTarget = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-src" "rust-std" "rust-analyzer" ];
          targets = [ "wasm32-wasip1" ];
        };

        # Scope nixpkgs overlay to this package
        craneLib = (crane.mkLib pkgs).overrideToolchain rustWithWasiTarget;

        zestty = craneLib.buildPackage {
          src = craneLib.cleanCargoSource (craneLib.path ./.);
          cargoExtraArgs = "--target wasm32-wasip1";
          doCheck = false;
          postInstall = ''
            mkdir -p $out/bin/
            cp ${./zestty} $out/bin/zestty
          '';
        };
      in
      {
        checks = {
          inherit zestty;
        };

        packages.default = zestty;

        devShells.default = craneLib.devShell {
          # Inherit inputs from checks.
          checks = self.checks.${system};
          packages = with pkgs; [
            rustWithWasiTarget
          ];
        };
      }
    );
}
