use crate::{
    core::{KillOutput, Killable, KillableBuildable, ProcessId, Result},
    Config, Error,
};
use tracing::instrument;

const KERNEL_PROCESS_ID: u32 = 0;
const INIT_PROCESS_ID: u32 = 1;

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::get_available_max_process_id;

    #[test]
    fn test_validate_process_id_kernel() {
        let result = validate_process_id(KERNEL_PROCESS_ID, 100);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Invalid process id: 0. Reason: Not allowed to kill kernel process"
        );
    }

    #[test]
    fn test_validate_process_id_init() {
        let result = validate_process_id(INIT_PROCESS_ID, 100);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Invalid process id: 1. Reason: Not allowed to kill init process"
        );
    }

    #[test]
    fn test_validate_process_id_too_large() {
        let result = validate_process_id(101, 100);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Invalid process id: 101. Reason: Process id is too large. process id: 101, available max process id: 100"
        );
    }

    #[test]
    fn test_validate_process_id_ok() {
        let result = validate_process_id(100, 100);
        assert!(result.is_ok());
    }

    #[test]
    fn kii_sigterm() {
        let target_process_id = get_available_max_process_id();
        let kill_output =
            kill(target_process_id, nix::sys::signal::Signal::SIGTERM).expect("Failed to kill");
        match kill_output {
            KillOutput::Killed { process_id: _ } => {
                panic!("This should not happen");
            }
            KillOutput::MaybeAlreadyTerminated { process_id, source } => {
                assert_eq!(process_id, target_process_id);
                assert_eq!(source.to_string(), "Unix error: ESRCH: No such process");
            }
        }
    }

    #[test]
    fn kii_sigkill() {
        let target_process_id = get_available_max_process_id();
        let kill_output =
            kill(target_process_id, nix::sys::signal::Signal::SIGKILL).expect("Failed to kill");
        match kill_output {
            KillOutput::Killed { process_id: _ } => {
                panic!("This should not happen");
            }
            KillOutput::MaybeAlreadyTerminated { process_id, source } => {
                assert_eq!(process_id, target_process_id);
                assert_eq!(source.to_string(), "Unix error: ESRCH: No such process");
            }
        }
    }

    #[test]
    fn kii_sigint() {
        let target_process_id = get_available_max_process_id();
        let kill_output =
            kill(target_process_id, nix::sys::signal::Signal::SIGINT).expect("Failed to kill");
        match kill_output {
            KillOutput::Killed { process_id: _ } => {
                panic!("This should not happen");
            }
            KillOutput::MaybeAlreadyTerminated { process_id, source } => {
                assert_eq!(process_id, target_process_id);
                assert_eq!(source.to_string(), "Unix error: ESRCH: No such process");
            }
        }
    }
}
