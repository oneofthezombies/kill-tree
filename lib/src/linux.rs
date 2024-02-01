use crate::common::{ProcessInfo, TreeKillable, TreeKiller};
use nix::errno::Errno;
use nix::sys::signal::{kill, Signal};
use nix::unistd::Pid;
use std::error::Error;
use std::fs;

const SWAPPER_PROCESS_ID: u32 = 0;
const INIT_PROCESS_ID: u32 = 1;

/// decimal 4194304
const AVAILABLE_MAX_PROCESS_ID: u32 = 0x400000;

impl TreeKillable for TreeKiller {
    fn kill_tree(&self) -> Result<Vec<u32>, Box<dyn Error>> {
        self.validate_pid()?;
        let signal = self.parse_signal()?;
        let process_infos = self.get_process_infos()?;
        let process_id_map = self.get_process_id_map(&process_infos, |_| false);
        let process_ids_to_kill = self.get_process_ids_to_kill(&process_id_map);
        for &process_id in process_ids_to_kill.iter().rev() {
            self.kill(process_id, signal)?;
        }
        Ok(process_ids_to_kill)
    }
}

impl TreeKiller {
    fn validate_pid(&self) -> Result<(), Box<dyn Error>> {
        self.validate_pid_with_available_max(AVAILABLE_MAX_PROCESS_ID)
    }

    fn get_process_infos(&self) -> Result<Vec<ProcessInfo>, Box<dyn Error>> {
        let mut process_infos = Vec::new();
        for entry in fs::read_dir("/proc")? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                if let Some(file_name) = path.file_name().and_then(|s| s.to_str()) {
                    if let Ok(process_id) = file_name.parse::<u32>() {
                        let status_path = path.join("status");
                        if status_path.exists() {
                            let status = fs::read_to_string(status_path)?;
                            let mut parent_process_id = None;
                            for line in status.lines() {
                                if line.starts_with("PPid:") {
                                    if let Some(parent_process_id_str) =
                                        line.split_whitespace().nth(1)
                                    {
                                        if let Ok(parent_process_id_value) =
                                            parent_process_id_str.parse::<u32>()
                                        {
                                            parent_process_id = Some(parent_process_id_value);
                                        }
                                    }
                                    break;
                                }
                            }
                            if let Some(parent_process_id) = parent_process_id {
                                process_infos.push(ProcessInfo {
                                    process_id,
                                    parent_process_id,
                                });
                            }
                        }
                    }
                }
            }
        }
        Ok(process_infos)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{common::Config, kill_tree_with_config};
    use std::{process::Command, thread, time::Duration};

    #[test]
    fn process_id_max_plus_1() {
        let result = kill_tree_with_config(AVAILABLE_MAX_PROCESS_ID + 1, Config::default());
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Process id is too large. process id: 4194305, available max process id: 4194304"
        );
    }

    #[test]
    fn hello_world_with_invalid_signal() {
        let process = Command::new("node")
            .arg("../tests/resources/hello_world.mjs")
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
