use crate::{
    common::{TreeKillable, TreeKiller},
    KillResults, ProcessId,
};
use nix::{
    errno::Errno,
    sys::signal::{kill, Signal},
    unistd::Pid,
};
use std::error::Error;

const KERNEL_PROCESS_ID: u32 = 0;
const INIT_PROCESS_ID: u32 = 1;

impl TreeKillable for TreeKiller {
    fn kill_tree(&self) -> Result<KillResults, Box<dyn Error>> {
        let signal = self.parse_signal()?;
        self.kill_tree_impl(|_| false, |process_id| self.kill(process_id, signal))
    }
}

impl TreeKiller {
    pub(crate) fn validate_pid_with_available_max(
        &self,
        available_max: u32,
    ) -> Result<(), Box<dyn Error>> {
        match self.process_id {
            KERNEL_PROCESS_ID => Err(format!(
                "Not allowed to kill kernel process. process id: {}",
                self.process_id
            )
            .into()),
            INIT_PROCESS_ID => Err(format!(
                "Not allowed to kill init process. process id: {}",
                self.process_id
            )
            .into()),
            _ => {
                if self.process_id <= available_max {
                    Ok(())
                } else {
                    Err(format!(
                        "Process id is too large. process id: {}, available max process id: {}",
                        self.process_id, available_max
                    )
                    .into())
                }
            }
        }
    }

    pub(crate) fn parse_signal(&self) -> Result<Signal, Box<dyn Error>> {
        self.config
            .signal
            .as_str()
            .parse::<Signal>()
            .map_err(|e| e.into())
    }

    pub(crate) fn kill(
        &self,
        process_id: ProcessId,
        signal: Signal,
    ) -> Result<Option<Box<dyn Error>>, Box<dyn Error>> {
        kill(Pid::from_raw(process_id as i32), signal)
            .and(Ok(None))
            .or_else(|e| {
                // ESRCH: No such process.
                // This happens when the process has already terminated.
                // This is not an error.
                if e == Errno::ESRCH {
                    Ok(Some(e.into()))
                } else {
                    Err(e.into())
                }
            })
    }
}

#[cfg(test)]
mod tests {
    use std::process::Command;

    use crate::{common::Config, kill_tree_with_config};

    #[test]
    fn process_id_0() {
        let result = kill_tree_with_config(0, Config::default());
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Not allowed to kill kernel process. process id: 0"
        );
    }

    #[test]
    fn process_id_1() {
        let result = kill_tree_with_config(1, Config::default());
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Not allowed to kill init process. process id: 1"
        );
    }

    #[test]
    fn hello_world_with_invalid_signal() {
        let process = Command::new("node")
            .arg("../../tests/resources/hello_world.mjs")
            .spawn()
            .unwrap();
        let process_id = process.id();
        let result = kill_tree_with_config(
            process_id,
            Config {
                signal: "SIGINVALID".to_string(),
            },
        );
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "EINVAL: Invalid argument");
    }
}
