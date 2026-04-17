#![no_std]
#![no_main]
#![cfg_attr(
  any(
    target_arch = "powerpc64",
    target_arch = "powerpc",
    target_arch = "sparc64",
    target_arch = "mips64"
  ),
  feature(asm_experimental_arch)
)]

extern crate alloc;

use core::{arch::naked_asm, panic::PanicInfo};

use microfetch_alloc::BumpAllocator;
use microfetch_asm::{entry_rust, sys_exit, sys_write};
// Re-export libc replacement functions from asm crate
pub use microfetch_asm::{memcpy, memset, strlen};

#[cfg(target_arch = "x86_64")]
#[unsafe(no_mangle)]
#[unsafe(naked)]
unsafe extern "C" fn _start() {
  naked_asm!(
    "mov rdi, rsp",
    "and rsp, -16",
    "call {entry_rust}",
    "mov rdi, rax",
    "mov rax, 60",
    "syscall",
    entry_rust = sym entry_rust,
  );
}

#[cfg(target_arch = "x86")]
#[unsafe(no_mangle)]
#[unsafe(naked)]
unsafe extern "C" fn _start() {
  naked_asm!(
    "mov eax, esp",    // save original sp (argc/argv)
    "and esp, -16",    // align stack to 16 bytes
    "sub esp, 12",     // leave room so that the one-arg push keeps alignment
    "push eax",        // arg: initial stack pointer
    "call {entry_rust}",
    "mov ebx, eax",    // exit code -> first syscall arg
    "mov eax, 1",      // SYS_exit
    "int 0x80",
    entry_rust = sym entry_rust,
  );
}

#[cfg(target_arch = "aarch64")]
#[unsafe(no_mangle)]
#[unsafe(naked)]
unsafe extern "C" fn _start() {
  naked_asm!(
    "mov x0, sp",
    "mov x9, sp",
    "and x9, x9, #-16",
    "mov sp, x9",
    "bl {entry_rust}",
    "mov x0, x0",
    "mov x8, 93",
    "svc #0",
    entry_rust = sym entry_rust,
  );
}

#[cfg(target_arch = "riscv64")]
#[unsafe(no_mangle)]
#[unsafe(naked)]
unsafe extern "C" fn _start() {
  naked_asm!(
    "mv a0, sp",
    "andi sp, sp, -16",
    "call {entry_rust}",
    "mv a0, a0",
    "li a7, 93",
    "ecall",
    entry_rust = sym entry_rust,
  );
}

#[cfg(target_arch = "loongarch64")]
#[unsafe(no_mangle)]
#[unsafe(naked)]
unsafe extern "C" fn _start() {
  naked_asm!(
    "or $a0, $sp, $zero",
    "bstrins.d $sp, $zero, 3, 0",
    "bl {entry_rust}",
    "or $a0, $a0, $zero",
    "li.w $a7, 93",
    "syscall 0",
    entry_rust = sym entry_rust,
  );
}

#[cfg(target_arch = "s390x")]
#[unsafe(no_mangle)]
#[unsafe(naked)]
unsafe extern "C" fn _start() {
  naked_asm!(
    "lgr %r2, %r15",       // save original sp (argc/argv) as arg
    "aghi %r15, -160",     // allocate s390x mandatory stack frame
    "lghi %r0, -16",
    "ngr %r15, %r0",       // align stack to 16 bytes
    "brasl %r14, {entry_rust}",
    "lghi %r1, 1",         // SYS_exit
    "svc 0",
    entry_rust = sym entry_rust,
  );
}

// ELFv2 (ppc64le / new BE): entry point is code directly, no function
// descriptor needed.
#[cfg(all(target_arch = "powerpc64", target_abi = "elfv2"))]
#[unsafe(no_mangle)]
#[unsafe(naked)]
unsafe extern "C" fn _start() {
  naked_asm!(
    // Set up TOC (r2) for ELFv2 by computing from current PC
    "bl 0f",
    "0: mflr 12",
    "addis 2, 12, .TOC. - 0b@ha",
    "addi 2, 2, .TOC. - 0b@l",
    // Save sp as first arg, set up stack frame
    "mr 3, 1",
    "clrrdi 1, 1, 4",
    "stdu 1, -64(1)",
    "bl {entry_rust}",
    "nop",
    "li 0, 1",
    "sc",
    entry_rust = sym entry_rust,
  );
}

// ppc32 has no function descriptors; entry is direct code like ELFv2.
#[cfg(target_arch = "powerpc")]
#[unsafe(no_mangle)]
#[unsafe(naked)]
unsafe extern "C" fn _start() {
  naked_asm!(
    "mr 3, 1",          // first arg = initial sp (argc/argv)
    "clrrwi 1, 1, 4",   // align sp to 16 bytes
    "stwu 1, -64(1)",   // reserve a frame with back-chain
    "bl {entry_rust}",
    "li 0, 1",          // SYS_exit
    "sc",
    entry_rust = sym entry_rust,
  );
}

// On ELFv1, the ELF entry point must be a function descriptor
// in .opd whose first word is the real code address. We emit both manually via
// `global_asm` so the kernel reads the descriptor and jumps into `_start_impl`.
#[cfg(all(target_arch = "powerpc64", target_abi = "elfv1"))]
core::arch::global_asm!(
  ".section .opd, \"aw\"",
  ".balign 8",
  ".globl _start",
  ".type _start, @object",
  ".size _start, 24",
  "_start:",
  "  .quad _start_impl",
  "  .quad .TOC.@tocbase",
  "  .quad 0",
  ".previous",
  ".text",
  ".type _start_impl, @function",
  "_start_impl:",
  "  mr 3, 1",
  "  clrrdi 1, 1, 4",
  "  stdu 1, -64(1)",
  "  bl entry_rust",
  "  nop",
  "  li 0, 1",
  "  sc",
);

#[cfg(target_arch = "arm")]
#[unsafe(no_mangle)]
#[unsafe(naked)]
unsafe extern "C" fn _start() {
  naked_asm!(
    "mov r0, sp",          // first arg = original sp (argc/argv)
    "bic sp, sp, #7",      // align sp to 8 bytes (AAPCS)
    "bl {entry_rust}",
    "mov r7, #1",          // SYS_exit
    "svc #0",
    entry_rust = sym entry_rust,
  );
}

#[cfg(target_arch = "riscv32")]
#[unsafe(no_mangle)]
#[unsafe(naked)]
unsafe extern "C" fn _start() {
  naked_asm!(
    "mv a0, sp",           // first arg = original sp (argc/argv)
    "andi sp, sp, -16",    // align sp to 16 bytes
    "call {entry_rust}",
    "li a7, 93",           // SYS_exit
    "ecall",
    entry_rust = sym entry_rust,
  );
}

#[cfg(target_arch = "sparc64")]
#[unsafe(no_mangle)]
#[unsafe(naked)]
unsafe extern "C" fn _start() {
  naked_asm!(
    // SPARC v9: kernel sets %sp biased by -2047; argc is at
    // %sp + 2047 + 128 (bias + 128-byte register save area).
    // See glibc sysdeps/sparc/sparc64/start.S.
    "mov %g0, %fp",
    "add %sp, 2047+128, %o0",  // first arg = &argc
    "add %sp, 2047, %sp",      // unbias
    "sub %sp, 176, %sp",       // reserve register save area
    "and %sp, -16, %sp",       // align
    "sub %sp, 2047, %sp",      // rebias
    "call {entry_rust}",
    "nop",                     // delay slot
    "mov 1, %g1",              // SYS_exit
    "t 0x6d",
    entry_rust = sym entry_rust,
  );
}

#[cfg(target_arch = "mips64")]
#[unsafe(no_mangle)]
#[unsafe(naked)]
unsafe extern "C" fn _start() {
  naked_asm!(
    "move $a0, $sp",       // first arg = original sp (argc/argv)
    "daddiu $sp, $sp, -16",// reserve + keep 16-byte alignment (N64 ABI)
    "jal {entry_rust}",
    "nop",                 // delay slot
    "move $a0, $v0",       // exit code = entry_rust return value
    "li $v0, 5058",        // SYS_exit (5000 + 58)
    "syscall",
    entry_rust = sym entry_rust,
  );
}

// Global allocator
#[global_allocator]
static ALLOCATOR: BumpAllocator = BumpAllocator::new();

/// Main application entry point. Called by the asm crate's entry point
/// after setting up argc, argv, and envp.
///
/// # Safety
///
/// argv must be a valid pointer to an array of argc C strings.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn main(argc: i32, argv: *const *const u8) -> i32 {
  // Calculate envp from argv. On Linux, envp is right after argv on the stack
  // but I bet 12 cents that there will be at least one exception.
  let argc_usize = usize::try_from(argc).unwrap_or(0);
  let envp = unsafe { argv.add(argc_usize + 1) };

  // Initialize the environment pointer
  unsafe {
    microfetch_lib::init_env(envp);
  }

  // Run the main application logic
  match unsafe { microfetch_lib::run(argc, argv) } {
    Ok(()) => 0,
    Err(e) => {
      let msg = alloc::format!("Error: {e}\n");
      let _ = unsafe { sys_write(2, msg.as_ptr(), msg.len()) };
      1
    },
  }
}

#[cfg(not(test))]
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
  const PANIC_MSG: &[u8] = b"panic\n";
  unsafe {
    let _ = sys_write(2, PANIC_MSG.as_ptr(), PANIC_MSG.len());
    sys_exit(1)
  }
}

// Stubs for Rust exception handling
#[cfg(not(test))]
#[unsafe(no_mangle)]
const extern "C" fn rust_eh_personality() {}

#[cfg(not(test))]
#[unsafe(no_mangle)]
extern "C" fn _Unwind_Resume() -> ! {
  unsafe { sys_exit(1) }
}

// compiler_builtins emits `.ARM.exidx` entries that reference these even
// with panic=abort. libgcc/libunwind would normally resolve them; we're
// nostdlib, so we stub them. They're never called.
#[cfg(all(not(test), target_arch = "arm"))]
#[unsafe(no_mangle)]
extern "C" fn __aeabi_unwind_cpp_pr0() {}

#[cfg(all(not(test), target_arch = "arm"))]
#[unsafe(no_mangle)]
extern "C" fn __aeabi_unwind_cpp_pr1() {}
