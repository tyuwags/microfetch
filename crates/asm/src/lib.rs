//! Incredibly fast syscall wrappers for using inline assembly. Serves the
//! purposes of completely bypassing Rust's standard library in favor of
//! handwritten Assembly. Is this a good idea? No. Is it fast? Yeah, but only
//! marginally. Either way it serves a purpose and I will NOT accept criticism.
//! What do you mean I wasted two whole hours to make the program only 100µs
//! faster?
//!
//! Supports `x86_64` and `aarch64` architectures. Riscv support will be
//! implemented when and ONLY WHEN I can be bothered to work on it.

use std::io;

/// Direct syscall to open a file
///
/// # Returns
///
/// File descriptor or -1 on error
///
/// # Safety
///
/// The caller must ensure:
///
/// - `path` points to a valid null-terminated C string
/// - The pointer remains valid for the duration of the syscall
#[inline]
#[must_use]
pub unsafe fn sys_open(path: *const u8, flags: i32) -> i32 {
  #[cfg(target_arch = "x86_64")]
  unsafe {
    let fd: i64;
    std::arch::asm!(
      "syscall",
      in("rax") 2i64,  // SYS_open
      in("rdi") path,
      in("rsi") flags,
      in("rdx") 0i32,  // mode (not used for reading)
      lateout("rax") fd,
      lateout("rcx") _,
      lateout("r11") _,
      options(nostack)
    );
    #[allow(clippy::cast_possible_truncation)]
    {
      fd as i32
    }
  }
  #[cfg(target_arch = "aarch64")]
  unsafe {
    let fd: i64;
    std::arch::asm!(
      "svc #0",
      in("x8") 56i64,  // SYS_openat
      in("x0") -100i32,  // AT_FDCWD
      in("x1") path,
      in("x2") flags,
      in("x3") 0i32,  // mode
      lateout("x0") fd,
      options(nostack)
    );
    #[allow(clippy::cast_possible_truncation)]
    {
      fd as i32
    }
  }
  #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
  {
    compile_error!("Unsupported architecture for inline assembly syscalls");
  }
}

/// Direct syscall to read from a file descriptor
///
/// # Returns
///
/// Number of bytes read or -1 on error
///
/// # Safety
///
/// The caller must ensure that:
///
/// - `buf` points to a valid writable buffer of at least `count` bytes
/// - `fd` is a valid open file descriptor
#[inline]
pub unsafe fn sys_read(fd: i32, buf: *mut u8, count: usize) -> isize {
  #[cfg(target_arch = "x86_64")]
  unsafe {
    let ret: i64;
    std::arch::asm!(
      "syscall",
      in("rax") 0i64,  // SYS_read
      in("rdi") fd,
      in("rsi") buf,
      in("rdx") count,
      lateout("rax") ret,
      lateout("rcx") _,
      lateout("r11") _,
      options(nostack)
    );

    #[allow(clippy::cast_possible_truncation)]
    {
      ret as isize
    }
  }

  #[cfg(target_arch = "aarch64")]
  unsafe {
    let ret: i64;
    std::arch::asm!(
      "svc #0",
      in("x8") 63i64,  // SYS_read
      in("x0") fd,
      in("x1") buf,
      in("x2") count,
      lateout("x0") ret,
      options(nostack)
    );

    #[allow(clippy::cast_possible_truncation)]
    {
      ret as isize
    }
  }

  #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
  {
    compile_error!("Unsupported architecture for inline assembly syscalls");
  }
}

/// Direct syscall to write to a file descriptor
///
/// # Returns
///
/// Number of bytes written or -1 on error
///
/// # Safety
///
/// The caller must ensure that:
///
/// - `buf` points to a valid readable buffer of at least `count` bytes
/// - `fd` is a valid open file descriptor
#[inline]
#[must_use]
pub unsafe fn sys_write(fd: i32, buf: *const u8, count: usize) -> isize {
  #[cfg(target_arch = "x86_64")]
  unsafe {
    let ret: i64;
    std::arch::asm!(
      "syscall",
      in("rax") 1i64,  // SYS_write
      in("rdi") fd,
      in("rsi") buf,
      in("rdx") count,
      lateout("rax") ret,
      lateout("rcx") _,
      lateout("r11") _,
      options(nostack)
    );

    #[allow(clippy::cast_possible_truncation)]
    {
      ret as isize
    }
  }

  #[cfg(target_arch = "aarch64")]
  unsafe {
    let ret: i64;
    std::arch::asm!(
      "svc #0",
      in("x8") 64i64,  // SYS_write
      in("x0") fd,
      in("x1") buf,
      in("x2") count,
      lateout("x0") ret,
      options(nostack)
    );

    #[allow(clippy::cast_possible_truncation)]
    {
      ret as isize
    }
  }

  #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
  {
    compile_error!("Unsupported architecture for inline assembly syscalls");
  }
}

/// Direct syscall to close a file descriptor
///
/// # Safety
///
/// The caller must ensure that `fd` is a valid open file descriptor
#[inline]
#[must_use]
pub unsafe fn sys_close(fd: i32) -> i32 {
  #[cfg(target_arch = "x86_64")]
  unsafe {
    let ret: i64;
    std::arch::asm!(
      "syscall",
      in("rax") 3i64,  // SYS_close
      in("rdi") fd,
      lateout("rax") ret,
      lateout("rcx") _,
      lateout("r11") _,
      options(nostack)
    );
    #[allow(clippy::cast_possible_truncation)]
    {
      ret as i32
    }
  }

  #[cfg(target_arch = "aarch64")]
  unsafe {
    let ret: i64;
    std::arch::asm!(
      "svc #0",
      in("x8") 57i64,  // SYS_close
      in("x0") fd,
      lateout("x0") ret,
      options(nostack)
    );
    #[allow(clippy::cast_possible_truncation)]
    {
      ret as i32
    }
  }

  #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
  {
    compile_error!("Unsupported architecture for inline assembly syscalls");
  }
}

/// Raw buffer for the `uname(2)` syscall.
///
/// Linux ABI hasfive fields of `[i8; 65]`: sysname, nodename, release, version,
/// machine. The `domainname` field (GNU extension, `[i8; 65]`) follows but is
/// not used, nor any useful to us here.
#[repr(C)]
#[allow(dead_code)]
pub struct UtsNameBuf {
  pub sysname:    [i8; 65],
  pub nodename:   [i8; 65],
  pub release:    [i8; 65],
  pub version:    [i8; 65],
  pub machine:    [i8; 65],
  pub domainname: [i8; 65], // GNU extension, included for correct struct size
}

/// Direct `uname(2)` syscall
///
/// # Returns
///
/// 0 on success, negative on error
///
/// # Safety
///
/// The caller must ensure that `buf` points to a valid `UtsNameBuf`.
#[inline]
#[allow(dead_code)]
pub unsafe fn sys_uname(buf: *mut UtsNameBuf) -> i32 {
  #[cfg(target_arch = "x86_64")]
  unsafe {
    let ret: i64;
    std::arch::asm!(
      "syscall",
      in("rax") 63i64,  // SYS_uname
      in("rdi") buf,
      lateout("rax") ret,
      lateout("rcx") _,
      lateout("r11") _,
      options(nostack)
    );

    #[allow(clippy::cast_possible_truncation)]
    {
      ret as i32
    }
  }

  #[cfg(target_arch = "aarch64")]
  unsafe {
    let ret: i64;
    std::arch::asm!(
      "svc #0",
      in("x8") 160i64,  // SYS_uname
      in("x0") buf,
      lateout("x0") ret,
      options(nostack)
    );
    #[allow(clippy::cast_possible_truncation)]
    {
      ret as i32
    }
  }

  #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
  {
    compile_error!("Unsupported architecture for inline assembly syscalls");
  }
}

/// Raw buffer for the `statfs(2)` syscall.
///
/// Linux ABI (`x86_64` and `aarch64`): the fields we use are at the same
/// offsets on both architectures. Only the fields needed for disk usage are
/// declared; the remainder of the 120-byte struct is covered by `_pad`.
#[repr(C)]
pub struct StatfsBuf {
  pub f_type:    i64,
  pub f_bsize:   i64,
  pub f_blocks:  u64,
  pub f_bfree:   u64,
  pub f_bavail:  u64,
  pub f_files:   u64,
  pub f_ffree:   u64,
  pub f_fsid:    [i32; 2],
  pub f_namelen: i64,
  pub f_frsize:  i64,
  pub f_flags:   i64,

  #[allow(clippy::pub_underscore_fields, reason = "This is not a public API")]
  pub _pad: [i64; 4],
}

/// Direct `statfs(2)` syscall
///
/// # Returns
///
/// 0 on success, negative errno on error
///
/// # Safety
///
/// The caller must ensure that:
///
/// - `path` points to a valid null-terminated string
/// - `buf` points to a valid `StatfsBuf`
#[inline]
pub unsafe fn sys_statfs(path: *const u8, buf: *mut StatfsBuf) -> i32 {
  #[cfg(target_arch = "x86_64")]
  unsafe {
    let ret: i64;
    std::arch::asm!(
      "syscall",
      in("rax") 137i64,  // SYS_statfs
      in("rdi") path,
      in("rsi") buf,
      lateout("rax") ret,
      lateout("rcx") _,
      lateout("r11") _,
      options(nostack)
    );

    #[allow(clippy::cast_possible_truncation)]
    {
      ret as i32
    }
  }

  #[cfg(target_arch = "aarch64")]
  unsafe {
    let ret: i64;
    std::arch::asm!(
      "svc #0",
      in("x8") 43i64,  // SYS_statfs
      in("x0") path,
      in("x1") buf,
      lateout("x0") ret,
      options(nostack)
    );

    #[allow(clippy::cast_possible_truncation)]
    {
      ret as i32
    }
  }

  #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
  {
    compile_error!("Unsupported architecture for inline assembly syscalls");
  }
}

/// Read entire file using direct syscalls. This avoids libc overhead and can be
/// significantly faster for small files.
///
/// # Errors
///
/// Returns an error if the file cannot be opened or read
#[inline]
pub fn read_file_fast(path: &str, buffer: &mut [u8]) -> io::Result<usize> {
  const O_RDONLY: i32 = 0;

  // We use stack-allocated buffer for null-terminated path. The maximum
  // is 256 bytes.
  let path_bytes = path.as_bytes();
  if path_bytes.len() >= 256 {
    return Err(io::Error::new(io::ErrorKind::InvalidInput, "Path too long"));
  }

  let mut path_buf = [0u8; 256];
  path_buf[..path_bytes.len()].copy_from_slice(path_bytes);
  // XXX: Already zero-terminated since array is initialized to zeros

  unsafe {
    let fd = sys_open(path_buf.as_ptr(), O_RDONLY);
    if fd < 0 {
      return Err(io::Error::last_os_error());
    }

    let bytes_read = sys_read(fd, buffer.as_mut_ptr(), buffer.len());
    let _ = sys_close(fd);

    if bytes_read < 0 {
      return Err(io::Error::last_os_error());
    }

    #[allow(clippy::cast_sign_loss)]
    {
      Ok(bytes_read as usize)
    }
  }
}
