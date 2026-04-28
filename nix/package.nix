{
  lib,
  makeRustPlatform,
  rust-bin,
  llvm,
}: let
  pname = "microfetch";
  toml = (lib.importTOML ../Cargo.toml).workspace.package;
  inherit (toml) version;

  toolchain = rust-bin.stable.latest;
  rustWithToolchain = makeRustPlatform {
    cargo = toolchain.minimal;
    rustc = toolchain.minimal;
  };
in
  rustWithToolchain.buildRustPackage.override {inherit (llvm) stdenv;} {
    inherit pname version;
    src = let
      fs = lib.fileset;
      s = ../.;
    in
      fs.toSource {
        root = s;
        fileset = fs.unions [
          (s + /.cargo)
          (s + /crates)
          (s + /microfetch)
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
