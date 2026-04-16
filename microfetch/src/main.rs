#![no_std]
#![no_main]

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
