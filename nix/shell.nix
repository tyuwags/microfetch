{
  mkShell,
  cargo,
  rustc,
  mold,
  clang,
  rust-analyzer,
  rustfmt,
  clippy,
  taplo,
  gnuplot,
}:
mkShell {
  name = "microfetch";
  strictDeps = true;
  nativeBuildInputs = [
    cargo
    rustc
    mold
    clang

    rust-analyzer
    (rustfmt.override {asNightly = true;})
    clippy
    taplo

    gnuplot # for Criterion.rs plots
  ];
}
