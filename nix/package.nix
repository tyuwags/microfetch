{
  lib,
  rustPlatform,
  llvm,
}: let
  pname = "microfetch";
  toml = (lib.importTOML ../Cargo.toml).workspace.package;
  inherit (toml) version;
in
  rustPlatform.buildRustPackage.override {inherit (llvm) stdenv;} (finalAttrs: {
    __structuredAttrs = true;

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

    cargoLock.lockFile = "${finalAttrs.src}/Cargo.lock";
    enableParallelBuilding = true;
    buildNoDefaultFeatures = true;
    doCheck = false;
    strictDeps = true;

    meta = {
      description = "Microscopic fetch script in Rust, for NixOS systems";
      homepage = "https://github.com/NotAShelf/microfetch";
      license = lib.licenses.gpl3Only;
      platforms = lib.platforms.linux;
      maintainers = [lib.maintainers.NotAShelf];
      mainProgram = "microfetch";
    };
  })
