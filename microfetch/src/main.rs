#![no_std]
#![no_main]

extern crate alloc;

use core::panic::PanicInfo;

use microfetch_alloc::BumpAllocator;
// Re-export libc replacement functions from asm crate
pub use microfetch_asm::{memcpy, memset, strlen};
use microfetch_asm::{sys_exit, sys_write};

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
