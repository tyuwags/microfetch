use alloc::string::String;

use crate::{Error, syscall::read_file_fast, system::write_u64};

/// Gets CPU model name (trimmed), or empty string if unavailable.
#[cfg_attr(feature = "hotpath", hotpath::measure)]
pub fn get_cpu_name() -> String {
  get_model_name().unwrap_or_default()
}

/// Gets CPU core/thread info as a string.
///
/// Format: `{cores} cores ({p}p/{e}e), {threads} threads` on hybrid Intel,
/// `{cores} cores, {threads} threads` otherwise.
///
/// # Errors
///
/// Returns an error if the thread count cannot be determined.
#[cfg_attr(feature = "hotpath", hotpath::measure)]
pub fn get_cpu_cores() -> Result<String, Error> {
  let threads = get_thread_count()?;
  let cores = get_core_count(threads);

  let mut result = String::new();

  write_u64(&mut result, u64::from(cores));
  result.push_str(" cores");

  if let Some((p, e)) = get_pe_cores() {
    result.push_str(" (");
    write_u64(&mut result, u64::from(p));
    result.push_str("p/");
    write_u64(&mut result, u64::from(e));
    result.push_str("e)");
  }

  if threads != cores {
    result.push_str(", ");
    write_u64(&mut result, u64::from(threads));
    result.push_str(" threads");
  }

  Ok(result)
}

/// Count online threads via `sched_getaffinity(2)`.
fn get_thread_count() -> Result<u32, Error> {
  let mut mask = [0u8; 128];
  let ret = unsafe {
    crate::syscall::sys_sched_getaffinity(0, mask.len(), mask.as_mut_ptr())
  };
  if ret < 0 {
    return Err(Error::from_raw_os_error(-ret));
  }

  #[allow(clippy::cast_sign_loss)]
  let bytes = ret as usize;
  let mut count = 0u32;
  for &byte in &mask[..bytes] {
    count += byte.count_ones();
  }
  Ok(count)
}

/// Derive physical core count from thread count and topology.
fn get_core_count(threads: u32) -> u32 {
  let Some(smt_width) =
    count_cpulist("/sys/devices/system/cpu/cpu0/topology/thread_siblings_list")
  else {
    return threads;
  };
  if smt_width == 0 {
    return threads;
  }
  threads / smt_width
}

/// Detect P-core and E-core counts via sysfs PMU device files, which is done
/// by reading `/sys/devices/cpu_core/cpus` and `/sys/devices/cpu_atom/cpus`.
fn get_pe_cores() -> Option<(u32, u32)> {
  let p = count_cpulist("/sys/devices/cpu_core/cpus")?;
  let e = count_cpulist("/sys/devices/cpu_atom/cpus").unwrap_or(0);
  if p > 0 || e > 0 { Some((p, e)) } else { None }
}

/// Parse a cpulist file and count listed CPUs.
fn count_cpulist(path: &str) -> Option<u32> {
  let mut buf = [0u8; 64];
  let n = read_file_fast(path, &mut buf).ok()?;
  let data = &buf[..n];

  let mut count = 0u32;
  let mut i = 0;
  while i < data.len() {
    // Parse start number
    let start = parse_num(data, &mut i);
    if i < data.len() && data[i] == b'-' {
      i += 1;
      let end = parse_num(data, &mut i);
      // The Kernel always emits ascending ranges, so end is always >= start
      // https://github.com/torvalds/linux/blob/v6.19/lib/vsprintf.c#L1276-L1303
      count += end - start + 1;
    } else {
      count += 1;
    }
    // Skip comma or newline
    if i < data.len() && (data[i] == b',' || data[i] == b'\n') {
      i += 1;
    }
  }
  Some(count)
}

/// Parse a decimal number from a byte slice, advancing the index.
fn parse_num(data: &[u8], i: &mut usize) -> u32 {
  let mut n = 0u32;
  while *i < data.len() && data[*i].is_ascii_digit() {
    n = n * 10 + u32::from(data[*i] - b'0');
    *i += 1;
  }
  n
}

/// Read CPU frequency in MHz. Tries sysfs first, then cpuinfo fields.
fn get_cpu_freq_mhz() -> Option<u32> {
  // Try sysfs cpuinfo_max_freq (in kHz)
  let mut buf = [0u8; 32];
  if let Ok(n) = read_file_fast(
    "/sys/devices/system/cpu/cpu0/cpufreq/cpuinfo_max_freq",
    &mut buf,
  ) {
    let mut khz = 0u32;
    for &b in &buf[..n] {
      if b.is_ascii_digit() {
        khz = khz * 10 + u32::from(b - b'0');
      }
    }
    if khz > 0 {
      return Some(khz / 1000);
    }
  }
  // Fall back to cpuinfo fields
  let mut buf2 = [0u8; 4096];
  let n = read_file_fast("/proc/cpuinfo", &mut buf2).ok()?;
  let data = &buf2[..n];
  for key in &[
    b"cpu MHz" as &[u8],
    b"cpu MHz dynamic",
    b"cpu MHz static",
    b"CPU MHz",
  ] {
    if let Some(val) = extract_field(data, key) {
      // Parse integer part of the MHz value (e.g. "5200.00" -> 5200)
      let mut mhz = 0u32;
      for &b in val.as_bytes() {
        if b == b'.' {
          break;
        }
        if b.is_ascii_digit() {
          mhz = mhz * 10 + u32::from(b - b'0');
        }
      }
      if mhz > 0 {
        return Some(mhz);
      }
    }
  }
  None
}

/// Parse CPU model name from `/proc/cpuinfo` and append frequency.
fn get_model_name() -> Option<String> {
  let mut buf = [0u8; 2048];
  let n = read_file_fast("/proc/cpuinfo", &mut buf).ok()?;
  let data = &buf[..n];

  for key in &[
    b"model name" as &[u8],
    b"Model Name",
    b"uarch",
    b"isa",
    b"cpu",
    b"machine",
    b"vendor_id",
  ] {
    if let Some(val) = extract_field(data, key) {
      let trimmed = trim(val);
      if !trimmed.is_empty() {
        let mut name = String::from(trimmed);
        if let Some(mhz) = get_cpu_freq_mhz() {
          name.push_str(" @ ");
          // Round to nearest 0.01 GHz, then split so carries (e.g. 1999 MHz)
          // roll into the integer part instead of overflowing the fraction.
          let rounded_centesimal = (mhz + 5) / 10;
          let ghz_int = rounded_centesimal / 100;
          let ghz_frac = rounded_centesimal % 100;
          write_u64(&mut name, u64::from(ghz_int));
          name.push('.');
          if ghz_frac < 10 {
            name.push('0');
          }
          write_u64(&mut name, u64::from(ghz_frac));
          name.push_str(" GHz");
        }
        return Some(name);
      }
    }
  }

  None
}

/// Extract value of first occurrence of `key` in cpuinfo.
fn extract_field<'a>(data: &'a [u8], key: &[u8]) -> Option<&'a str> {
  let mut i = 0;
  while i < data.len() {
    let remaining = &data[i..];
    let eol = remaining
      .iter()
      .position(|&b| b == b'\n')
      .unwrap_or(remaining.len());
    let line = &remaining[..eol];

    if line.starts_with(key) {
      let mut p = key.len();
      while p < line.len() && (line[p] == b'\t' || line[p] == b' ') {
        p += 1;
      }
      if p < line.len() && line[p] == b':' {
        p += 1;
        while p < line.len() && line[p] == b' ' {
          p += 1;
        }
        return core::str::from_utf8(&line[p..]).ok();
      }
    }

    i += eol + 1;
  }
  None
}

/// Strip noise from model names.
fn trim(name: &str) -> &str {
  let b = name.as_bytes();
  let mut end = b.len();

  while end > 0 && b[end - 1].is_ascii_whitespace() {
    end -= 1;
  }

  if end >= 10 && &b[end - 10..end] == b" Processor" {
    end -= 10;
  } else if end >= 4 && &b[end - 4..end] == b" CPU" {
    end -= 4;
  }
  while end > 0 && b[end - 1].is_ascii_whitespace() {
    end -= 1;
  }

  if end >= 3 && &b[end - 3..end] == b"(R)" {
    end -= 3;
  } else if end >= 4
    && (&b[end - 4..end] == b"(TM)" || &b[end - 4..end] == b"(tm)")
  {
    end -= 4;
  }
  while end > 0 && b[end - 1].is_ascii_whitespace() {
    end -= 1;
  }

  if end > 7 && &b[end - 5..end] == b"-Core" {
    let mut p = end - 5;
    while p > 0 && b[p - 1].is_ascii_digit() {
      p -= 1;
    }
    if p > 0 && b[p - 1] == b' ' {
      end = p - 1;
    }
  }
  while end > 0 && b[end - 1].is_ascii_whitespace() {
    end -= 1;
  }

  let mut start = 0;
  while start < end && b[start].is_ascii_whitespace() {
    start += 1;
  }

  &name[start..end]
}
