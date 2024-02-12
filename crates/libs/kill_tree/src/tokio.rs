use crate::core::{tokio::ProcessInfosProvidable, Config, Outputs, ProcessId, Result};

#[cfg(target_os = "linux")]
use crate::linux as imp;
#[cfg(target_os = "macos")]
use crate::macos as imp;
#[cfg(windows)]
use crate::windows as imp;

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
/// use kill_tree::{get_available_max_process_id, tokio::kill_tree, Result};
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///     let _ = kill_tree(get_available_max_process_id()).await?;
///     Ok(())
/// }
/// ```
///
/// # Errors
///
/// ## `InvalidProcessId`
/// Returns the process ID of the kernel or system, or if greater than the available maximum process ID.  
///
/// ## `InvalidCast`
/// Returned internally when an invalid type conversion occurs during a system API call.  
/// This is an error that should not occur under normal circumstances.  
///
/// ## `InvalidProcEntry`
/// Returned when inquiry, or parsing within the Linux `/proc/` path fails.  
///
/// ## `Io`
/// Returned when access within the Linux `/proc/` path fails.  
///
/// ## `Windows`
/// Returned when the `Win32` API used internally fails.  
///
/// ## `Unix`
/// Returned when the `libc` API used internally fails.  
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
/// use kill_tree::{get_available_max_process_id, tokio::kill_tree_with_config, Config, Result};
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///     let config = Config {
///         signal: String::from("SIGKILL"),
///         ..Default::default()
///     };
///     let _ = kill_tree_with_config(get_available_max_process_id(), &config).await?;
///     Ok(())
/// }
/// ```
///
/// Kills all children __except the target process__.  
/// ```
/// use kill_tree::{get_available_max_process_id, tokio::kill_tree_with_config, Config, Result};
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///     let config = Config {
///         include_target: false,
///         ..Default::default()
///     };
///     let _ = kill_tree_with_config(get_available_max_process_id(), &config).await?;
///     Ok(())
/// }
/// ```
///
/// # Errors
///
/// ## `InvalidProcessId`
/// Returns the process ID of the kernel or system, or if greater than the available maximum process ID.  
///
/// ## `InvalidCast`
/// Returned internally when an invalid type conversion occurs during a system API call.  
/// This is an error that should not occur under normal circumstances.  
///
/// ## `InvalidProcEntry`
/// Returned when inquiry, or parsing within the Linux `/proc/` path fails.  
///
/// ## `Io`
/// Returned when access within the Linux `/proc/` path fails.  
///
/// ## `Windows`
/// Returned when the `Win32` API used internally fails.  
///
/// ## `Unix`
/// Returned when the `libc` API used internally fails.  
pub async fn kill_tree_with_config(process_id: ProcessId, config: &Config) -> Result<Outputs> {
    imp::validate_process_id(process_id)?;
    let process_infos_provider = imp::tokio::ProcessInfosProvider {};
    let process_infos = process_infos_provider.get_process_infos().await?;
    crate::common::kill_tree_internal(process_id, config, process_infos)
}
