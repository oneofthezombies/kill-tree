use crate::{
    core::{KillOutput, ProcessId, Result},
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

fn parse_process_id_sign(process_id: ProcessId) -> Result<i32> {
    let process_id_sign = i32::try_from(process_id).map_err(|e| Error::InvalidCast {
        reason: "Failed to cast process id to i32".into(),
        source: e,
    })?;
    Ok(process_id_sign)
}

fn parse_kill_result(result: nix::Result<()>, process_id: ProcessId) -> Result<KillOutput> {
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

fn kill(process_id_sign: i32, signal: nix::sys::signal::Signal) -> nix::Result<()> {
    nix::sys::signal::kill(nix::unistd::Pid::from_raw(process_id_sign), signal)
}

pub(crate) mod blocking {
    use super::*;

    struct Killer {
        signal: nix::sys::signal::Signal,
    }

    impl crate::blocking::Killable for Killer {
        fn kill(&self, process_id: ProcessId) -> Result<KillOutput> {
            crate::unix::blocking::kill(process_id, self.signal)
        }
    }

    pub(crate) fn new_killer(config: &Config) -> Result<impl crate::blocking::Killable> {
        let signal = config.signal.parse()?;
        Ok(Killer { signal })
    }

    #[instrument]
    pub(crate) fn kill(
        process_id: ProcessId,
        signal: nix::sys::signal::Signal,
    ) -> Result<KillOutput> {
        let process_id_sign = parse_process_id_sign(process_id)?;
        let result = crate::unix::kill(process_id_sign, signal);
        parse_kill_result(result, process_id)
    }
}

pub(crate) mod tokio {
    use async_trait::async_trait;

    use super::*;

    #[derive(Clone)]
    struct Killer {
        signal: nix::sys::signal::Signal,
    }

    #[async_trait]
    impl crate::tokio::Killable for Killer {
        async fn kill(&self, process_id: ProcessId) -> Result<KillOutput> {
            crate::unix::tokio::kill(process_id, self.signal).await
        }
    }

    pub(crate) fn new_killer(config: &Config) -> Result<impl crate::tokio::Killable> {
        let signal = config.signal.parse()?;
        Ok(Killer { signal })
    }

    #[instrument]
    pub(crate) async fn kill(
        process_id: ProcessId,
        signal: nix::sys::signal::Signal,
    ) -> Result<KillOutput> {
        let process_id_sign = parse_process_id_sign(process_id)?;
        let result =
            ::tokio::task::spawn_blocking(move || crate::unix::kill(process_id_sign, signal))
                .await?;
        parse_kill_result(result, process_id)
    }
}

// #[cfg(test)]
// mod tests {
//     use std::process::Command;

//     use crate::{kill_tree, kill_tree_with_signal};

//     #[tokio::test]
//     async fn process_id_0() {
//         let result = kill_tree(0).await;
//         assert!(result.is_err());
//         assert_eq!(
//             result.unwrap_err().to_string(),
//             "Not allowed to kill kernel process. process id: 0"
//         );
//     }

//     #[tokio::test]
//     async fn process_id_1() {
//         let result = kill_tree(1).await;
//         assert!(result.is_err());
//         assert_eq!(
//             result.unwrap_err().to_string(),
//             "Not allowed to kill init process. process id: 1"
//         );
//     }

//     #[tokio::test]
//     async fn hello_world_with_invalid_signal() {
//         let process = Command::new("node")
//             .arg("../../../tests/resources/hello_world.mjs")
//             .spawn()
//             .unwrap();
//         let process_id = process.id();
//         let result = kill_tree_with_signal(process_id, "SIGINVALID").await;
//         assert!(result.is_err());
//         assert_eq!(result.unwrap_err().to_string(), "EINVAL: Invalid argument");
//     }
// }
