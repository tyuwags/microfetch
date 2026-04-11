//! Simple bump allocator for `no_std` environments. Uses a statically allocated
//! 32KB buffer and provides O(1) allocation with no deallocation support
//! (memory is never freed).
#![no_std]
use core::{
  alloc::{GlobalAlloc, Layout},
  cell::UnsafeCell,
  ptr::null_mut,
};

/// Default heap size is 32KB, should be plenty for Microfetch. Technically it
/// can be invoked with more (or less) depending on our needs but I am quite
/// sure 32KB is more than enough.
pub const DEFAULT_HEAP_SIZE: usize = 32 * 1024;

/// A simple bump allocator that never frees memory.
///
/// This allocator maintains a static buffer and a bump pointer. Allocations are
/// fast (just bump the pointer), but memory is never reclaimed. While you might
/// be inclined to point out that this is ugly, it's suitable for a short-lived
/// program with bounded memory usage.
pub struct BumpAllocator<const N: usize = DEFAULT_HEAP_SIZE> {
  heap: UnsafeCell<[u8; N]>,
  next: UnsafeCell<usize>,
}

// SAFETY: BumpAllocator is thread-safe because it uses UnsafeCell
// and the allocator is only used in single-threaded contexts (i.e., no_std).
unsafe impl<const N: usize> Sync for BumpAllocator<N> {}

impl<const N: usize> BumpAllocator<N> {
  /// Creates a new bump allocator with the specified heap size.
  #[must_use]
  pub const fn new() -> Self {
    Self {
      heap: UnsafeCell::new([0; N]),
      next: UnsafeCell::new(0),
    }
  }

  /// Returns the number of bytes currently allocated.
  #[must_use]
  pub fn used(&self) -> usize {
    // SAFETY: We're just reading the value, and this is only called
    // in single-threaded contexts.
    unsafe { *self.next.get() }
  }

  /// Returns the total heap size.
  #[must_use]
  pub const fn capacity(&self) -> usize {
    N
  }

  /// Returns the number of bytes remaining.
  #[must_use]
  pub fn remaining(&self) -> usize {
    N - self.used()
  }
}

impl<const N: usize> Default for BumpAllocator<N> {
  fn default() -> Self {
    Self::new()
  }
}

unsafe impl<const N: usize> GlobalAlloc for BumpAllocator<N> {
  unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
    unsafe {
      let next = self.next.get();
      let heap = self.heap.get();

      // Align the current position
      let align = layout.align();
      let start = (*next + align - 1) & !(align - 1);
      let end = start + layout.size();

      if end > N {
        // Out of memory
        null_mut()
      } else {
        *next = end;
        (*heap).as_mut_ptr().add(start)
      }
    }
  }

  unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
    // Bump allocator doesn't support deallocation
    // Memory is reclaimed when the program exits
  }
}

/// Static bump allocator instance with 32KB heap.
///
/// # Example
///
/// Use this with `#[global_allocator]` in your binary:
///
///
/// ```rust,ignore
/// #[global_allocator]
/// static ALLOCATOR: BumpAllocator = BumpAllocator::new();
/// ```
pub type BumpAlloc = BumpAllocator<DEFAULT_HEAP_SIZE>;
