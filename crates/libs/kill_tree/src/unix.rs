use crate::{
    common::{
        self,
        single::{self},
        tree, Impl,
    },
    ProcessId,
};
use nix::{
    errno::Errno,
    sys::signal::{self, Signal},
    unistd::Pid,
};
use tracing::instrument;

const KERNEL_PROCESS_ID: u32 = 0;
const INIT_PROCESS_ID: u32 = 1;

pub(crate) fn validate_process_id(
    process_id: ProcessId,
    available_max: ProcessId,
) -> common::Result<()> {
    match process_id {
        KERNEL_PROCESS_ID => {
            Err(format!("Not allowed to kill kernel process. process id: {process_id}").into())
        }
        INIT_PROCESS_ID => {
            Err(format!("Not allowed to kill init process. process id: {process_id}").into())
        }
        _ => {
            if process_id <= available_max {
                Ok(())
            } else {
                Err(format!(
                    "Process id is too large. process id: {process_id}, available max process id: {available_max}"
                )
                .into())
            }
        }
    }
}

#[instrument]
fn kill(process_id: ProcessId, signal: Signal) -> common::Result<single::Output> {
    let process_id_sign = i32::try_from(process_id)?;
    let result = signal::kill(Pid::from_raw(process_id_sign), signal);
    match result {
        Ok(()) => Ok(single::Output::Killed { process_id }),
        Err(e) => {
            // ESRCH: No such process.
            // This happens when the process has already terminated.
            // This treat as success.
            if e == Errno::ESRCH {
                return Ok(single::Output::MaybeAlreadyTerminated {
                    process_id,
                    reason: e.into(),
                });
            }

            Err(e.into())
        }
    }
}

impl Impl {
    pub(crate) async fn kill_tree(&self) -> common::Result<tree::Outputs> {
        let signal = self.signal.parse::<Signal>()?;
        self.kill_tree_impl(|_| false, move |process_id| kill(process_id, signal))
            .await
    }
}

#[cfg(test)]
mod tests {
    use std::process::Command;

    use crate::{kill_tree, kill_tree_with_signal};

    #[tokio::test]
    async fn process_id_0() {
        let result = kill_tree(0).await;
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Not allowed to kill kernel process. process id: 0"
        );
    }

    #[tokio::test]
    async fn process_id_1() {
        let result = kill_tree(1).await;
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Not allowed to kill init process. process id: 1"
        );
    }

    #[tokio::test]
    async fn hello_world_with_invalid_signal() {
        let process = Command::new("node")
            .arg("../../../tests/resources/hello_world.mjs")
            .spawn()
            .unwrap();
        let process_id = process.id();
        let result = kill_tree_with_signal(process_id, "SIGINVALID").await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "EINVAL: Invalid argument");
    }
}
