pub mod colors;
pub mod desktop;
pub mod release;
pub mod system;
pub mod uptime;

use std::{
  ffi::CStr,
  io::{self, Cursor, Write},
  mem::MaybeUninit,
};

pub use microfetch_asm as syscall;
pub use microfetch_asm::{
  StatfsBuf,
  SysInfo,
  UtsNameBuf,
  read_file_fast,
  sys_close,
  sys_open,
  sys_read,
  sys_statfs,
  sys_sysinfo,
  sys_uname,
  sys_write,
};

/// Wrapper for `utsname` with safe accessor methods
pub struct UtsName(UtsNameBuf);

impl UtsName {
  /// Calls `uname(2)` syscall and returns a `UtsName` wrapper
  ///
  /// # Errors
  ///
  /// Returns an error if the `uname` syscall fails
  pub fn uname() -> Result<Self, std::io::Error> {
    let mut uts = MaybeUninit::uninit();
    if unsafe { sys_uname(uts.as_mut_ptr()) } != 0 {
      return Err(std::io::Error::last_os_error());
    }
    Ok(Self(unsafe { uts.assume_init() }))
  }

  #[must_use]
  pub const fn nodename(&self) -> &CStr {
    unsafe { CStr::from_ptr(self.0.nodename.as_ptr().cast()) }
  }

  #[must_use]
  pub const fn sysname(&self) -> &CStr {
    unsafe { CStr::from_ptr(self.0.sysname.as_ptr().cast()) }
  }

  #[must_use]
  pub const fn release(&self) -> &CStr {
    unsafe { CStr::from_ptr(self.0.release.as_ptr().cast()) }
  }

  #[must_use]
  pub const fn machine(&self) -> &CStr {
    unsafe { CStr::from_ptr(self.0.machine.as_ptr().cast()) }
  }
}

// Struct to hold all the fields we need in order to print the fetch. This
// helps avoid Clippy warnings about argument count, and makes it slightly
// easier to pass data around. Though, it is not like we really need to.
struct Fields {
  user_info:      String,
  os_name:        String,
  kernel_version: String,
  shell:          String,
  uptime:         String,
  desktop:        String,
  memory_usage:   String,
  storage:        String,
  colors:         String,
}

#[cfg_attr(feature = "hotpath", hotpath::measure)]
fn print_system_info(
  fields: &Fields,
) -> Result<(), Box<dyn std::error::Error>> {
  let Fields {
    user_info,
    os_name,
    kernel_version,
    shell,
    uptime,
    desktop,
    memory_usage,
    storage,
    colors,
  } = fields;

  let cyan = colors::COLORS.cyan;
  let blue = colors::COLORS.blue;
  let reset = colors::COLORS.reset;

  let mut buf = [0u8; 2048];
  let mut cursor = Cursor::new(&mut buf[..]);

  write!(
    cursor,
    "
    {blue}     ▟█▖    {cyan}▝█▙ ▗█▛         {user_info} ~{reset}
    {blue}  ▗▄▄▟██▄▄▄▄▄{cyan}▝█▙█▛  {blue}▖       {cyan}  {blue}System{reset}        {os_name}
    {blue}  ▀▀▀▀▀▀▀▀▀▀▀▘{cyan}▝██  {blue}▟█▖      {cyan}  {blue}Kernel{reset}        {kernel_version}
    {cyan}     ▟█▛       {cyan}▝█▘{blue}▟█▛       {cyan}  {blue}Shell{reset}         {shell}
    {cyan}▟█████▛          {blue}▟█████▛    {cyan}  {blue}Uptime{reset}        {uptime}
    {cyan}   ▟█▛{blue}▗█▖       {blue}▟█▛         {cyan}  {blue}Desktop{reset}       {desktop}
    {cyan}  ▝█▛  {blue}██▖{cyan}▗▄▄▄▄▄▄▄▄▄▄▄      {cyan}󰍛  {blue}Memory{reset}        {memory_usage}
    {cyan}   ▝  {blue}▟█▜█▖{cyan}▀▀▀▀▀██▛▀▀▘      {cyan}󱥎  {blue}Storage (/){reset}   {storage}
    {blue}     ▟█▘ ▜█▖    {cyan}▝█▛         {cyan}  {blue}Colors{reset}        {colors}\n\n"
  )?;

  let len =
    usize::try_from(cursor.position()).expect("cursor position fits usize");
  // Direct syscall to avoid stdout buffering allocation
  let written = unsafe { sys_write(1, buf.as_ptr(), len) };
  if written < 0 {
    return Err(io::Error::last_os_error().into());
  }
  #[allow(clippy::cast_sign_loss)] // non-negative verified by the guard above
  if written as usize != len {
    return Err(
      io::Error::new(io::ErrorKind::WriteZero, "partial write to stdout")
        .into(),
    );
  }
  Ok(())
}

/// Main entry point for microfetch - can be called by the binary crate
/// or by other consumers of the library
///
/// # Errors
///
/// Returns an error if any system call fails
#[cfg_attr(feature = "hotpath", hotpath::main)]
pub fn run() -> Result<(), Box<dyn std::error::Error>> {
  if Some("--version") == std::env::args().nth(1).as_deref() {
    println!("Microfetch {}", env!("CARGO_PKG_VERSION"));
  } else {
    let utsname = UtsName::uname()?;
    let fields = Fields {
      user_info:      system::get_username_and_hostname(&utsname),
      os_name:        release::get_os_pretty_name()?,
      kernel_version: release::get_system_info(&utsname),
      shell:          system::get_shell(),
      desktop:        desktop::get_desktop_info(),
      uptime:         uptime::get_current()?,
      memory_usage:   system::get_memory_usage()?,
      storage:        system::get_root_disk_usage()?,
      colors:         colors::print_dots(),
    };
    print_system_info(&fields)?;
  }

  Ok(())
}
