{
  description = "Microscopic fetch script in Rust, for NixOS systems";
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs?ref=nixos-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = {
    self,
    nixpkgs,
    rust-overlay,
  }: let
    systems = ["x86_64-linux" "aarch64-linux"];
    forEachSystem = nixpkgs.lib.genAttrs systems;
    pkgsForEach = system: nixpkgs.legacyPackages.${system}.extend rust-overlay.overlays.default;
  in {
    packages = forEachSystem (system: let
      pkgs = pkgsForEach system;
    in {
      default = self.packages.${system}.microfetch;
      microfetch = pkgs.callPackage ./nix/package.nix {};
    });

    devShells = forEachSystem (system: {
      default = (pkgsForEach system).callPackage ./nix/shell.nix {};
    });
  };
}
