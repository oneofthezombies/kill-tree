use crate::core::{Config, Error, Outputs, ProcessId, Result};

#[cfg(target_os = "linux")]
use crate::linux as imp;
#[cfg(target_os = "macos")]
use crate::macos as imp;
#[cfg(windows)]
use crate::windows as imp;

impl From<tokio::task::JoinError> for Error {
    fn from(e: tokio::task::JoinError) -> Self {
        Error::TokioJoin(e)
    }
}

/// Returns the max available process ID.
/// # Platform-specifics
/// ## Windows
/// In hexadecimal, 0xFFFFFFFF.  
/// In decimal, 4294967295.  
/// But actually process IDs are generated as multiples of 4.  
///
/// ## Linux
/// In hexadecimal, 0x400000.  
/// In decimal, 4194304.  
///
/// ## Macos
/// In decimal, 99998.  
///
/// # Examples
///
/// ```
/// use kill_tree::blocking::get_available_max_process_id;
///
/// #[cfg(windows)]
/// assert!(get_available_max_process_id() == 0xFFFF_FFFF);
///
/// #[cfg(target_os = "linux")]
/// assert!(get_available_max_process_id() == 0x0040_0000);
///
/// #[cfg(target_os = "macos")]
/// assert!(get_available_max_process_id() == 99998);
/// ```
pub fn get_available_max_process_id() -> u32 {
    crate::common::get_available_max_process_id()
}

/// Kills the target process and all of its children recursively.  
/// # Platform-specifics
///
/// ## Windows
/// Internally, `TerminateProcess` from `Win32` is used.  
///
/// ## Linux, Macos
/// Internally, `kill` from `libc` is used.  
/// Default signal is `SIGTERM`.  
/// If you want to send a different `signal` instead of `SIGTERM`, use `kill_tree_with_config`.  
///
/// # Examples
/// ```
/// use kill_tree::{
///     tokio::{get_available_max_process_id, kill_tree},
///     Result,
/// };
///
/// fn main() -> Result<()> {
///     let _ = kill_tree(get_available_max_process_id())?;
///     Ok(())
/// }
/// ```
pub async fn kill_tree(process_id: ProcessId) -> Result<Outputs> {
    kill_tree_with_config(process_id, &Config::default()).await
}

/// Kills the target process and all of its children recursively using the given `Config`.  
/// # Platform-specifics
///
/// ## Windows
/// Internally, `TerminateProcess` from `Win32` is used.  
///
/// ## Linux, Macos
/// Internally, `kill` from `libc` is used.  
///
/// # Examples
///
/// Kill processes using the `SIGKILL` signal.  
/// ```
/// use kill_tree::{
///     blocking::{get_available_max_process_id, kill_tree_with_config},
///     Config, Result,
/// };
///
/// fn main() -> Result<()> {
///     let config = Config {
///         signal: String::from("SIGKILL"),
///         ..Default::default()
///     };
///     let _ = kill_tree_with_config(get_available_max_process_id(), &config)?;
///     Ok(())
/// }
/// ```
///
/// Kills all children __except the target process__.  
/// ```
/// use kill_tree::{
///     blocking::{get_available_max_process_id, kill_tree_with_config},
///     Config, Result,
/// };
///
/// fn main() -> Result<()> {
///     let config = Config {
///         include_target: false,
///         ..Default::default()
///     };
///     let _ = kill_tree_with_config(get_available_max_process_id(), &config)?;
///     Ok(())
/// }
/// ```
pub async fn kill_tree_with_config(process_id: ProcessId, config: &Config) -> Result<Outputs> {
    imp::validate_process_id(process_id)?;
    let process_infos = imp::tokio::get_process_infos().await?;
    crate::common::kill_tree_internal(process_id, config, process_infos)
}
