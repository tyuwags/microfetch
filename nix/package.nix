{
  lib,
  rustPlatform,
  llvm,
}: let
  pname = "microfetch";
  toml = (lib.importTOML ../Cargo.toml).workspace.package;
  inherit (toml) version;
in
  rustPlatform.buildRustPackage.override {inherit (llvm) stdenv;} {
    inherit pname version;
    src = let
      fs = lib.fileset;
      s = ../.;
    in
      fs.toSource {
        root = s;
        fileset = fs.unions [
          (s + /crates)
          (s + /microfetch)
          (s + /.cargo)
          (s + /scripts/ld-wrapper)
          (s + /Cargo.lock)
          (s + /Cargo.toml)
        ];
      };

    cargoLock.lockFile = ../Cargo.lock;
    enableParallelBuilding = true;
    buildNoDefaultFeatures = true;
    doCheck = false;

    meta = {
      description = "Microscopic fetch script in Rust, for NixOS systems";
      homepage = "https://github.com/NotAShelf/microfetch";
      license = lib.licenses.gpl3Only;
      maintainers = [lib.maintainers.NotAShelf];
      mainProgram = "microfetch";
    };
  }
