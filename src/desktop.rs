use std::{ffi::OsStr, fmt::Write};

#[must_use]
#[cfg_attr(feature = "hotpath", hotpath::measure)]
pub fn get_desktop_info() -> String {
  let desktop_os = std::env::var_os("XDG_CURRENT_DESKTOP");
  let session_os = std::env::var_os("XDG_SESSION_TYPE");

  let desktop_raw = desktop_os
    .as_deref()
    .and_then(OsStr::to_str)
    .unwrap_or("Unknown");
  let desktop_str = desktop_raw.strip_prefix("none+").unwrap_or(desktop_raw);

  let session_raw = session_os.as_deref().and_then(OsStr::to_str).unwrap_or("");
  let backend_str = if session_raw.is_empty() {
    "Unknown"
  } else {
    session_raw
  };

  // Pre-calculate capacity: desktop_len + " (" + backend_len + ")"
  // Capitalize first char needs temporary allocation only if backend exists
  let mut result =
    String::with_capacity(desktop_str.len() + backend_str.len() + 3);
  result.push_str(desktop_str);
  result.push_str(" (");

  // Capitalize first character of backend
  if let Some(first_char) = backend_str.chars().next() {
    let _ = write!(result, "{}", first_char.to_ascii_uppercase());
    result.push_str(&backend_str[first_char.len_utf8()..]);
  }

  result.push(')');
  result
}
