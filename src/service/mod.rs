#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;

#[cfg(target_os = "linux")]
pub use linux::*;
#[cfg(target_os = "macos")]
pub use macos::*;

// Fallback for unsupported platforms
#[cfg(not(any(target_os = "macos", target_os = "linux")))]
compile_error!("sysmon service management is only supported on macOS and Linux");
