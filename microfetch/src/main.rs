#![no_std]
#![no_main]

extern crate alloc;

use microfetch_alloc::BumpAlloc;
use microfetch_asm::sys_write;
#[cfg(not(test))]
use {core::panic::PanicInfo, microfetch_asm::sys_exit};

#[global_allocator]
static ALLOCATOR: BumpAlloc = BumpAlloc::new();

/// Receives argc and argv directly. The C runtime will call this after
/// initializing the environment. Cool right?
///
/// # Safety
///
/// argv must be a valid pointer to an array of argc C strings.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn main(argc: i32, argv: *const *const u8) -> i32 {
  // SAFETY: argc and argv are provided by the C runtime and are valid
  unsafe {
    match microfetch_lib::run(argc, argv) {
      Ok(()) => 0,
      Err(e) => {
        // Print error message to stderr (fd 2)
        let msg = alloc::format!("Error: {e}\n");
        let _ = sys_write(2, msg.as_ptr(), msg.len());
        1
      },
    }
  }
}

#[cfg(not(test))]
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
  // Write "panic" to stderr and exit
  const PANIC_MSG: &[u8] = b"panic\n";
  unsafe {
    let _ = sys_write(2, PANIC_MSG.as_ptr(), PANIC_MSG.len());
    sys_exit(1)
  }
}

// FIXME: Stubs for Rust exception handling symbols needed when using alloc with
// panic=abort These are normally provided by the unwinding runtime, but we're
// using panic=abort. I don't actually think this is the correct approach, but I
// cannot think of anything better.

#[cfg(not(test))]
#[unsafe(no_mangle)]
const extern "C" fn rust_eh_personality() {}

#[cfg(not(test))]
#[unsafe(no_mangle)]
extern "C" fn _Unwind_Resume() -> ! {
  unsafe { sys_exit(1) }
}
