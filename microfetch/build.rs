fn main() {
  // These flags only apply to the microfetch binary, not to proc-macro crates
  // or other host-compiled artifacts.

  // No C runtime, we provide _start ourselves
  println!("cargo:rustc-link-arg-bin=microfetch=-nostartfiles");
  // Fully static, no dynamic linker, no .interp/.dynsym/.dynamic overhead
  println!("cargo:rustc-link-arg-bin=microfetch=-static");
  // Remove unreferenced input sections
  println!("cargo:rustc-link-arg-bin=microfetch=-Wl,--gc-sections");
  // Strip all symbol table entries
  println!("cargo:rustc-link-arg-bin=microfetch=-Wl,--strip-all");
  // Omit the .note.gnu.build-id section
  println!("cargo:rustc-link-arg-bin=microfetch=-Wl,--build-id=none");
  // Disable RELRO (removes relro_padding)
  println!("cargo:rustc-link-arg-bin=microfetch=-Wl,-z,norelro");
}
