use alloc::string::String;

use crate::getenv_str;

#[must_use]
#[cfg_attr(feature = "hotpath", hotpath::measure)]
pub fn get_desktop_info() -> String {
  let desktop_raw = getenv_str("XDG_CURRENT_DESKTOP").unwrap_or("Unknown");
  let session_raw = getenv_str("XDG_SESSION_TYPE").unwrap_or("");

  let desktop_str = desktop_raw.strip_prefix("none+").unwrap_or(desktop_raw);

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
  if let Some(first_byte) = backend_str.as_bytes().first() {
    // Convert first byte to uppercase if it's ASCII lowercase
    let upper = if first_byte.is_ascii_lowercase() {
      (first_byte - b'a' + b'A') as char
    } else {
      *first_byte as char
    };
    result.push(upper);
    result.push_str(&backend_str[1..]);
  }

  result.push(')');
  result
}
