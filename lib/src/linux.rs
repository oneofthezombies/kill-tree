use crate::common::{ProcessInfo, TreeKiller};
use std::error::Error;
use std::fs;

/// decimal 4194304
const AVAILABLE_MAX_PROCESS_ID: u32 = 0x400000;

impl TreeKiller {
    pub(crate) fn validate_pid(&self) -> Result<(), Box<dyn Error>> {
        self.validate_pid_with_available_max(AVAILABLE_MAX_PROCESS_ID)
    }

    pub(crate) fn get_process_infos(&self) -> Result<Vec<ProcessInfo>, Box<dyn Error>> {
        let mut process_infos = Vec::new();
        for entry in fs::read_dir("/proc")? {
            let entry = entry?;
            let path = entry.path();
            // if path is not a directory then continue

            if path.is_dir() {
                if let Some(file_name) = path.file_name().and_then(|s| s.to_str()) {
                    if let Ok(process_id) = file_name.parse::<u32>() {
                        let status_path = path.join("status");
                        if status_path.exists() {
                            let status = fs::read_to_string(status_path)?;
                            let mut parent_process_id = None;
                            let mut name = None;
                            for line in status.lines() {
                                if parent_process_id.is_some() && name.is_some() {
                                    break;
                                }
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
                                } else if line.starts_with("Name:") {
                                    name =
                                        Some(line.split_whitespace().nth(1).unwrap().to_string());
                                }
                            }

                            // if parent_process_id is None then assume it is 0
                            let parent_process_id = parent_process_id.unwrap_or(0);

                            // if name is None then assume it is "unknown"
                            let name = name.unwrap_or_else(|| "unknown".to_string());

                            process_infos.push(ProcessInfo {
                                process_id,
                                parent_process_id,
                                name,
                            });
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

    #[test]
    fn process_id_max_plus_1() {
        let result = kill_tree_with_config(AVAILABLE_MAX_PROCESS_ID + 1, Config::default());
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Process id is too large. process id: 4194305, available max process id: 4194304"
        );
    }
}
