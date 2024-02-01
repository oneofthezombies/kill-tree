use crate::common::TreeKiller;
use nix::{
    errno::Errno,
    sys::signal::{kill, Signal},
    unistd::Pid,
};
use std::error::Error;

const KERNEL_PROCESS_ID: u32 = 0;
const INIT_PROCESS_ID: u32 = 1;

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

    pub(crate) fn kill(&self, process_id: u32, signal: Signal) -> Result<(), Box<dyn Error>> {
        kill(Pid::from_raw(process_id as i32), signal).or_else(|e| {
            // ESRCH: No such process.
            // This happens when the process has already terminated.
            // This is not an error.
            if e == Errno::ESRCH {
                Ok(())
            } else {
                Err(e.into())
            }
        })
    }
}

#[cfg(test)]
mod tests {
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
}
