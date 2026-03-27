use std::{ffi::OsStr, fmt::Write as _, io, mem::MaybeUninit};

use crate::{
  UtsName,
  colors::COLORS,
  syscall::{StatfsBuf, read_file_fast, sys_statfs},
};

#[must_use]
#[cfg_attr(feature = "hotpath", hotpath::measure)]
pub fn get_username_and_hostname(utsname: &UtsName) -> String {
  let username_os = std::env::var_os("USER");
  let username = username_os
    .as_deref()
    .and_then(OsStr::to_str)
    .unwrap_or("unknown_user");
  let hostname = utsname.nodename().to_str().unwrap_or("unknown_host");

  let capacity = COLORS.yellow.len()
    + username.len()
    + COLORS.red.len()
    + 1
    + COLORS.green.len()
    + hostname.len()
    + COLORS.reset.len();
  let mut result = String::with_capacity(capacity);

  result.push_str(COLORS.yellow);
  result.push_str(username);
  result.push_str(COLORS.red);
  result.push('@');
  result.push_str(COLORS.green);
  result.push_str(hostname);
  result.push_str(COLORS.reset);

  result
}

#[must_use]
#[cfg_attr(feature = "hotpath", hotpath::measure)]
pub fn get_shell() -> String {
  let shell_os = std::env::var_os("SHELL");
  let shell = shell_os.as_deref().and_then(OsStr::to_str).unwrap_or("");
  let start = shell.rfind('/').map_or(0, |i| i + 1);
  if shell.is_empty() {
    "unknown_shell".into()
  } else {
    shell[start..].into()
  }
}

/// Gets the root disk usage information.
///
/// # Errors
///
/// Returns an error if the filesystem information cannot be retrieved.
#[cfg_attr(feature = "hotpath", hotpath::measure)]
#[allow(clippy::cast_precision_loss)]
pub fn get_root_disk_usage() -> Result<String, io::Error> {
  let mut vfs = MaybeUninit::<StatfsBuf>::uninit();
  let path = b"/\0";

  if unsafe { sys_statfs(path.as_ptr(), vfs.as_mut_ptr()) } != 0 {
    return Err(io::Error::last_os_error());
  }

  let vfs = unsafe { vfs.assume_init() };
  #[allow(clippy::cast_sign_loss)]
  let block_size = vfs.f_bsize as u64;
  let total_blocks = vfs.f_blocks;
  let available_blocks = vfs.f_bavail;

  let total_size = block_size * total_blocks;
  let used_size = total_size - (block_size * available_blocks);

  let total_size = total_size as f64 / (1024.0 * 1024.0 * 1024.0);
  let used_size = used_size as f64 / (1024.0 * 1024.0 * 1024.0);
  let usage = (used_size / total_size) * 100.0;

  let mut result = String::with_capacity(64);
  write!(
    result,
    "{used_size:.2} GiB / {total_size:.2} GiB ({cyan}{usage:.0}%{reset})",
    cyan = COLORS.cyan,
    reset = COLORS.reset,
  )
  .unwrap();

  Ok(result)
}

/// Fast integer parsing without stdlib overhead
#[inline]
fn parse_u64_fast(s: &[u8]) -> u64 {
  let mut result = 0u64;
  for &byte in s {
    if byte.is_ascii_digit() {
      result = result * 10 + u64::from(byte - b'0');
    } else {
      break;
    }
  }
  result
}

/// Gets the system memory usage information.
///
/// # Errors
///
/// Returns an error if `/proc/meminfo` cannot be read.
#[cfg_attr(feature = "hotpath", hotpath::measure)]
pub fn get_memory_usage() -> Result<String, io::Error> {
  #[cfg_attr(feature = "hotpath", hotpath::measure)]
  fn parse_memory_info() -> Result<(f64, f64), io::Error> {
    let mut total_memory_kb = 0u64;
    let mut available_memory_kb = 0u64;
    let mut buffer = [0u8; 1024];

    // Use fast syscall-based file reading
    let bytes_read = read_file_fast("/proc/meminfo", &mut buffer)?;
    let meminfo = &buffer[..bytes_read];

    // Fast scanning for MemTotal and MemAvailable
    let mut offset = 0;
    let mut found_total = false;
    let mut found_available = false;

    while offset < meminfo.len() && (!found_total || !found_available) {
      let remaining = &meminfo[offset..];

      // Find newline or end
      let line_end = remaining
        .iter()
        .position(|&b| b == b'\n')
        .unwrap_or(remaining.len());
      let line = &remaining[..line_end];

      if line.starts_with(b"MemTotal:") {
        // Skip "MemTotal:" and whitespace
        let mut pos = 9;
        while pos < line.len() && line[pos].is_ascii_whitespace() {
          pos += 1;
        }
        total_memory_kb = parse_u64_fast(&line[pos..]);
        found_total = true;
      } else if line.starts_with(b"MemAvailable:") {
        // Skip "MemAvailable:" and whitespace
        let mut pos = 13;
        while pos < line.len() && line[pos].is_ascii_whitespace() {
          pos += 1;
        }
        available_memory_kb = parse_u64_fast(&line[pos..]);
        found_available = true;
      }

      offset += line_end + 1;
    }

    #[allow(clippy::cast_precision_loss)]
    let total_gb = total_memory_kb as f64 / 1024.0 / 1024.0;
    #[allow(clippy::cast_precision_loss)]
    let available_gb = available_memory_kb as f64 / 1024.0 / 1024.0;
    let used_memory_gb = total_gb - available_gb;

    Ok((used_memory_gb, total_gb))
  }

  let (used_memory, total_memory) = parse_memory_info()?;
  #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
  let percentage_used = (used_memory / total_memory * 100.0).round() as u64;

  let mut result = String::with_capacity(64);
  write!(
    result,
    "{used_memory:.2} GiB / {total_memory:.2} GiB \
     ({cyan}{percentage_used}%{reset})",
    cyan = COLORS.cyan,
    reset = COLORS.reset,
  )
  .unwrap();

  Ok(result)
}
