use crate::{
    core::{KillOutput, Killable, ProcessId, Result},
    Config, Error,
};
use tracing::instrument;

const KERNEL_PROCESS_ID: u32 = 0;
const INIT_PROCESS_ID: u32 = 1;

impl From<nix::Error> for Error {
    fn from(error: nix::Error) -> Self {
        Self::Unix(error)
    }
}

pub(crate) fn validate_process_id(process_id: ProcessId, available_max: ProcessId) -> Result<()> {
    match process_id {
        KERNEL_PROCESS_ID => Err(Error::InvalidProcessId {
            process_id,
            reason: "Not allowed to kill kernel process".into(),
        }),
        INIT_PROCESS_ID => Err(Error::InvalidProcessId {
            process_id,
            reason: "Not allowed to kill init process".into(),
        }),
        _ => {
            if process_id <= available_max {
                Ok(())
            } else {
                Err(Error::InvalidProcessId {
                    process_id,
                    reason: format!(
                        "Process id is too large. process id: {process_id}, available max process id: {available_max}"
                    ),
                })
            }
        }
    }
}

#[instrument]
pub(crate) fn kill(process_id: ProcessId, signal: nix::sys::signal::Signal) -> Result<KillOutput> {
    let process_id_sign = i32::try_from(process_id).map_err(|e| Error::InvalidCast {
        reason: "Failed to cast process id to i32".into(),
        source: e,
    })?;
    let result = nix::sys::signal::kill(nix::unistd::Pid::from_raw(process_id_sign), signal);
    match result {
        Ok(()) => Ok(KillOutput::Killed { process_id }),
        Err(e) => {
            // ESRCH: No such process.
            // This happens when the process has already terminated.
            // This treat as success.
            if e == nix::errno::Errno::ESRCH {
                Ok(KillOutput::MaybeAlreadyTerminated {
                    process_id,
                    source: e.into(),
                })
            } else {
                Err(e.into())
            }
        }
    }
}

#[derive(Clone)]
pub(crate) struct Killer {
    signal: nix::sys::signal::Signal,
}

impl Killable for Killer {
    fn kill(&self, process_id: ProcessId) -> Result<KillOutput> {
        crate::unix::kill(process_id, self.signal)
    }
}

pub(crate) struct KillerBuilder {}

impl KillableBuildable for KillerBuilder {
    fn new_killable(&self, config: &Config) -> Result<Killer> {
        let signal = config.signal.parse()?;
        Ok(Killer { signal })
    }
}
