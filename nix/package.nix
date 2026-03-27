{
  lib,
  rustPlatform,
  llvm,
}: let
  toml = (lib.importTOML ../Cargo.toml).package;
  pname = toml.name;
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
          (fs.fileFilter (file: builtins.any file.hasExt ["rs"]) (s + /crates))
          (fs.fileFilter (file: builtins.any file.hasExt ["rs"]) (s + /microfetch))
          (s + /.cargo)
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
