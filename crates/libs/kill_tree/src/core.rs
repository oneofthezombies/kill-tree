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
    #[cfg(feature = "tokio")]
    TokioJoin(::tokio::task::JoinError),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::InvalidProcessId { process_id, reason } => {
                write!(f, "Invalid process id: {process_id}. Reason: {reason}")
            }
            Error::InvalidCast { source, reason } => {
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
            #[cfg(feature = "tokio")]
            Error::TokioJoin(e) => write!(f, "Tokio join error: {e}"),
        }
    }
}

impl std::error::Error for Error {}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
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
    use super::{Error, ProcessInfos, Result};

    impl From<::tokio::task::JoinError> for Error {
        fn from(e: ::tokio::task::JoinError) -> Self {
            Error::TokioJoin(e)
        }
    }

    pub(crate) trait ProcessInfosProvidable {
        async fn get_process_infos(&self) -> Result<ProcessInfos>;
    }
}
