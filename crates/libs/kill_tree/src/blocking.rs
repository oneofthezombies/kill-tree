use crate::core::{blocking::ProcessInfosProvidable, Config, Outputs, ProcessId, Result};

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
/// use kill_tree::{blocking::kill_tree, get_available_max_process_id, Result};
///
/// fn main() -> Result<()> {
///     let target_process_id = get_available_max_process_id(); // Replace with your target process ID.
///     let _ = kill_tree(target_process_id)?;
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
pub fn kill_tree(process_id: ProcessId) -> Result<Outputs> {
    kill_tree_with_config(process_id, &Config::default())
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
///     blocking::kill_tree_with_config, get_available_max_process_id, Config, Result,
/// };
///
/// fn main() -> Result<()> {
///     let target_process_id = get_available_max_process_id(); // Replace with your target process ID.
///     let config = Config {
///         signal: String::from("SIGKILL"),
///         ..Default::default()
///     };
///     let _ = kill_tree_with_config(target_process_id, &config)?;
///     Ok(())
/// }
/// ```
///
/// Kills all children __except the target process__.  
/// ```
/// use kill_tree::{
///     blocking::kill_tree_with_config, get_available_max_process_id, Config, Result,
/// };
///
/// fn main() -> Result<()> {
///     let target_process_id = get_available_max_process_id(); // Replace with your target process ID.
///     let config = Config {
///         include_target: false,
///         ..Default::default()
///     };
///     let _ = kill_tree_with_config(target_process_id, &config)?;
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
pub fn kill_tree_with_config(process_id: ProcessId, config: &Config) -> Result<Outputs> {
    imp::validate_process_id(process_id)?;
    let process_infos_provider = imp::blocking::ProcessInfosProvider {};
    let process_infos = process_infos_provider.get_process_infos()?;
    crate::common::kill_tree_internal(process_id, config, process_infos)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::get_available_max_process_id;

    #[cfg(windows)]
    #[test]
    fn kill_tree_available_max_process_id_windows() {
        let target_process_id = get_available_max_process_id();
        let result = kill_tree(target_process_id).expect("Failed to kill");
        assert_eq!(result.len(), 1);
        let output = &result[0];
        match output {
            crate::Output::Killed { .. } => {
                panic!("This should not happen");
            }
            crate::Output::MaybeAlreadyTerminated { process_id, source } => {
                assert_eq!(*process_id, target_process_id);
                assert_eq!(
                    source.to_string(),
                    "Windows error: The parameter is incorrect. (0x80070057)"
                );
            }
        }
    }

    #[cfg(unix)]
    #[test]
    fn kill_tree_available_max_process_id_unix() {
        let target_process_id = get_available_max_process_id();
        let result = kill_tree(target_process_id).expect("Failed to kill");
        assert_eq!(result.len(), 1);
        let output = &result[0];
        match output {
            crate::Output::Killed { .. } => {
                panic!("This should not happen");
            }
            crate::Output::MaybeAlreadyTerminated { process_id, source } => {
                assert_eq!(*process_id, target_process_id);
                assert_eq!(source.to_string(), "Unix error: ESRCH: No such process");
            }
        }
    }

    #[cfg(windows)]
    #[test]
    fn kill_tree_with_config_sigkill_available_max_process_id_windows() {
        let target_process_id = get_available_max_process_id();
        let config = Config {
            signal: String::from("SIGKILL"),
            ..Default::default()
        };
        let result = kill_tree_with_config(target_process_id, &config).expect("Failed to kill");
        assert_eq!(result.len(), 1);
        let output = &result[0];
        match output {
            crate::Output::Killed { .. } => {
                panic!("This should not happen");
            }
            crate::Output::MaybeAlreadyTerminated { process_id, source } => {
                assert_eq!(*process_id, target_process_id);
                assert_eq!(
                    source.to_string(),
                    "Windows error: The parameter is incorrect. (0x80070057)"
                );
            }
        }
    }

    #[cfg(unix)]
    #[test]
    fn kill_tree_with_config_sigkill_available_max_process_id_unix() {
        let target_process_id = get_available_max_process_id();
        let config = Config {
            signal: String::from("SIGKILL"),
            ..Default::default()
        };
        let result = kill_tree_with_config(target_process_id, &config).expect("Failed to kill");
        assert_eq!(result.len(), 1);
        let output = &result[0];
        match output {
            crate::Output::Killed { .. } => {
                panic!("This should not happen");
            }
            crate::Output::MaybeAlreadyTerminated { process_id, source } => {
                assert_eq!(*process_id, target_process_id);
                assert_eq!(source.to_string(), "Unix error: ESRCH: No such process");
            }
        }
    }

    #[test]
    fn kill_tree_with_config_include_target_false_available_max_process_id() {
        let target_process_id = get_available_max_process_id();
        let config = Config {
            include_target: false,
            ..Default::default()
        };
        let result = kill_tree_with_config(target_process_id, &config).expect("Failed to kill");
        assert_eq!(result.len(), 0);
    }
}
