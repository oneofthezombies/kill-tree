#[derive(Debug)]
pub enum Error {
    InvalidProcessId {
        process_id: ProcessId,
        reason: String,
    },
    InvalidCast {
        source: std::num::TryFromIntError,
        reason: String,
    },
    #[cfg(target_os = "windows")]
    Windows(windows::core::Error),
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
            #[cfg(target_os = "windows")]
            Error::Windows(e) => write!(f, "Windows error: {e}"),
        }
    }
}

impl std::error::Error for Error {}

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

pub(crate) enum KillOutput {
    Killed {
        process_id: ProcessId,
    },
    MaybeAlreadyTerminated {
        process_id: ProcessId,
        source: Error,
    },
}

pub(crate) type KillOutputs = Vec<KillOutput>;

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
