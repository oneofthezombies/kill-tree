use crate::common::{ProcessInfo, TreeKillable, TreeKiller};
use nix::sys::signal::{kill, Signal};
use nix::unistd::Pid;
use std::error::Error;
use std::fs;

const SWAPPER_PROCESS_ID: u32 = 0;
const INIT_PROCESS_ID: u32 = 1;
const AVAILABLE_MAX_PROCESS_ID: u32 = 0x400000;

impl TreeKillable for TreeKiller {
    fn kill_tree(&self) -> Result<(), Box<dyn Error>> {
        self.validate_pid()?;
        let signal = self.parse_signal()?;
        let process_infos = self.get_process_infos()?;
        let process_id_map = self.get_process_id_map(&process_infos, |_| false);
        let process_ids_to_kill = self.get_process_ids_to_kill(&process_id_map);
        for process_id in process_ids_to_kill.iter().rev() {
            self.kill(*process_id, signal)?;
        }
        Ok(())
    }
}

impl TreeKiller {
    fn validate_pid(&self) -> Result<(), Box<dyn Error>> {
        match self.process_id {
            SWAPPER_PROCESS_ID => Err(format!(
                "Not allowed to kill swapper process. process id: {}",
                self.process_id
            )
            .into()),
            INIT_PROCESS_ID => Err(format!(
                "Not allowed to kill init process. process id: {}",
                self.process_id
            )
            .into()),
            _ => {
                if self.process_id <= AVAILABLE_MAX_PROCESS_ID {
                    Ok(())
                } else {
                    Err(format!(
                        "Process id is too large. process id: {}, available max process id: {}",
                        self.process_id, AVAILABLE_MAX_PROCESS_ID
                    )
                    .into())
                }
            }
        }
    }

    fn parse_signal(&self) -> Result<Signal, Box<dyn Error>> {
        self.config
            .signal
            .as_str()
            .parse::<Signal>()
            .map_err(|e| e.into())
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

    fn kill(&self, process_id: u32, signal: Signal) -> Result<(), Box<dyn Error>> {
        kill(Pid::from_raw(process_id as i32), signal).map_err(|e| e.into())
    }
}
