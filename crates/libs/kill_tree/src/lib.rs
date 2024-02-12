mod common;
mod core;

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(unix)]
mod unix;
#[cfg(windows)]
mod windows;

#[cfg(feature = "blocking")]
pub mod blocking;
#[cfg(feature = "tokio")]
pub mod tokio;

pub use crate::core::{Config, Error, Output, Outputs, ParentProcessId, ProcessId, Result};
