mod common;
mod core;

pub use crate::core::{Config, Error, Output, Outputs, ParentProcessId, ProcessId, Result};

#[cfg(unix)]
mod unix;

#[cfg(windows)]
mod windows;

#[cfg(target_os = "linux")]
mod linux;

#[cfg(target_os = "macos")]
mod macos;

#[cfg(feature = "blocking")]
pub mod blocking;

#[cfg(feature = "tokio")]
pub mod tokio;
