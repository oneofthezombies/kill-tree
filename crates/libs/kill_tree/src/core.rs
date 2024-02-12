#[derive(Debug)]
pub enum Error {
    InvalidProcessId {
        process_id: ProcessId,
        reason: String,
    },
    InvalidCast {
        reason: String,
        source: std::num::TryFromIntError,
    },
    InvalidProcEntry {
        process_id: ProcessId,
        path: String,
        reason: String,
        source: Option<std::num::ParseIntError>,
    },
    Io(std::io::Error),
    #[cfg(windows)]
    Windows(windows::core::Error),
    #[cfg(unix)]
    Unix(nix::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::InvalidProcessId { process_id, reason } => {
                write!(f, "Invalid process id: {process_id}. Reason: {reason}")
            }
            Error::InvalidCast { reason, source } => {
                write!(f, "Invalid cast. Reason: {reason}. Source: {source}")
            }
            Error::InvalidProcEntry {
                process_id,
                path,
                reason,
                source,
            } => write!(
                f,
                "Invalid proc entry. Process id: {process_id}. Path: {path}. Reason: {reason}. Source: {source:?}"
            ),
            Error::Io(e) => write!(f, "I/O error: {e}"),
            #[cfg(windows)]
            Error::Windows(e) => write!(f, "Windows error: {e}"),
            #[cfg(unix)]
            Error::Unix(e) => write!(f, "Unix error: {e}"),
        }
    }
}

impl std::error::Error for Error {}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

#[cfg(windows)]
impl From<::windows::core::Error> for Error {
    fn from(error: ::windows::core::Error) -> Self {
        Self::Windows(error)
    }
}

#[cfg(unix)]
impl From<nix::Error> for Error {
    fn from(error: nix::Error) -> Self {
        Self::Unix(error)
    }
}

pub type Result<T> = std::result::Result<T, Error>;

pub type ProcessId = u32;
pub type ParentProcessId = ProcessId;

#[derive(Debug)]
pub(crate) struct ProcessInfo {
    pub(crate) process_id: ProcessId,
    pub(crate) parent_process_id: ParentProcessId,
    pub(crate) name: String,
}

pub(crate) type ChildProcessId = ProcessId;
pub(crate) type ChildProcessIds = Vec<ChildProcessId>;
pub(crate) type ProcessIds = Vec<ProcessId>;
pub(crate) type ProcessInfos = Vec<ProcessInfo>;
pub(crate) type ChildProcessIdMap = std::collections::HashMap<ProcessId, ChildProcessIds>;
pub(crate) type ProcessInfoMap = std::collections::HashMap<ProcessId, ProcessInfo>;
pub(crate) type ChildProcessIdMapFilter = fn(&ProcessInfo) -> bool;

pub(crate) trait Killable {
    fn kill(&self, process_id: ProcessId) -> Result<KillOutput>;
}

pub(crate) trait KillableBuildable {
    fn new_killable(&self, config: &Config) -> Result<impl Killable>;
}

pub(crate) enum KillOutput {
    Killed {
        process_id: ProcessId,
    },
    MaybeAlreadyTerminated {
        process_id: ProcessId,
        source: Error,
    },
}

#[derive(Debug)]
pub enum Output {
    Killed {
        process_id: ProcessId,
        parent_process_id: ParentProcessId,
        name: String,
    },
    MaybeAlreadyTerminated {
        process_id: ProcessId,
        source: Error,
    },
}

pub type Outputs = Vec<Output>;

#[derive(Debug)]
pub struct Config {
    pub signal: String,
    pub include_target: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            signal: "SIGTERM".to_string(),
            include_target: true,
        }
    }
}

#[cfg(feature = "blocking")]
pub(crate) mod blocking {
    use super::{ProcessInfos, Result};

    pub(crate) trait ProcessInfosProvidable {
        fn get_process_infos(&self) -> Result<ProcessInfos>;
    }
}

#[cfg(feature = "tokio")]
pub(crate) mod tokio {
    use super::{ProcessInfos, Result};

    pub(crate) trait ProcessInfosProvidable {
        async fn get_process_infos(&self) -> Result<ProcessInfos>;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_display_invalid_process_id() {
        let error = Error::InvalidProcessId {
            process_id: 0,
            reason: "reason".to_string(),
        };
        assert_eq!(
            format!("{}", error),
            "Invalid process id: 0. Reason: reason"
        );
    }

    #[test]
    fn error_display_invalid_cast() {
        let error = Error::InvalidCast {
            reason: "reason".to_string(),
            source: u32::try_from(-1).unwrap_err(),
        };
        assert_eq!(
            format!("{}", error),
            "Invalid cast. Reason: reason. Source: out of range integral type conversion attempted"
        );
    }

    #[test]
    fn error_display_invalid_proc_entry() {
        let error = Error::InvalidProcEntry {
            process_id: 0,
            path: "/proc/0".to_string(),
            reason: "reason".to_string(),
            source: Some("source".parse::<u32>().unwrap_err()),
        };
        assert_eq!(
            format!("{}", error),
            "Invalid proc entry. Process id: 0. Path: /proc/0. Reason: reason. Source: Some(ParseIntError { kind: InvalidDigit })"
        );
    }

    #[test]
    fn error_display_io() {
        let error = Error::Io(std::io::Error::new(std::io::ErrorKind::Other, "error"));
        assert_eq!(format!("{}", error), "I/O error: error");
    }

    #[cfg(windows)]
    #[test]
    fn error_display_windows() {
        let error = Error::Windows(windows::core::Error::OK);
        assert_eq!(
            format!("{}", error),
            "Windows error: The operation completed successfully. (0x00000000)"
        );
    }

    #[cfg(unix)]
    #[test]
    fn error_display_unix() {
        let error = Error::Unix(nix::Error::UnsupportedOperation);
        assert_eq!(format!("{}", error), "Unix error: UnsupportedOperation");
    }

    #[test]
    fn from_io_error() {
        let error = std::io::Error::new(std::io::ErrorKind::Other, "error");
        let error = Error::from(error);
        assert_eq!(format!("{}", error), "I/O error: error");
    }

    #[cfg(windows)]
    #[test]
    fn from_windows_error() {
        let error = windows::core::Error::OK;
        let error = Error::from(error);
        assert_eq!(
            format!("{}", error),
            "Windows error: The operation completed successfully. (0x00000000)"
        );
    }

    #[cfg(unix)]
    #[test]
    fn from_unix_error() {
        let error = nix::Error::UnsupportedOperation;
        let error = Error::from(error);
        assert_eq!(format!("{}", error), "Unix error: UnsupportedOperation");
    }

    #[test]
    fn default_config() {
        let config = Config::default();
        assert_eq!(config.signal, "SIGTERM");
        assert_eq!(config.include_target, true);
    }
}
