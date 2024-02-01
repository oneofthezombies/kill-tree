use crate::common::{ProcessInfo, TreeKillable, TreeKiller};
use std::{error::Error, ffi};
use windows::Win32::{
    Foundation::{CloseHandle, ERROR_NO_MORE_FILES, E_ACCESSDENIED},
    System::{
        Diagnostics::ToolHelp::{
            CreateToolhelp32Snapshot, Process32First, Process32Next, PROCESSENTRY32,
            TH32CS_SNAPPROCESS,
        },
        Threading::{OpenProcess, TerminateProcess, PROCESS_TERMINATE},
    },
};

/// process id of System Idle Process
const SYSTEM_IDLE_PROCESS_PROCESS_ID: u32 = 0;

/// process id of System
const SYSTEM_PROCESS_ID: u32 = 4;

impl TreeKillable for TreeKiller {
    fn kill_tree(&self) -> Result<Vec<u32>, Box<dyn Error>> {
        // self.config is not used on Windows platform yet
        let _ = self.config;
        self.validate_pid()?;
        let process_infos = self.get_process_infos()?;
        let process_id_map = self.get_process_id_map(&process_infos, |process_info| {
            // this process is System Idle Process
            process_info.parent_process_id == process_info.process_id
        });
        let process_ids_to_kill = self.get_process_ids_to_kill(&process_id_map);
        for process_id in process_ids_to_kill.iter().rev() {
            self.terminate_process(*process_id)?;
        }
        Ok(process_ids_to_kill)
    }
}

impl TreeKiller {
    fn validate_pid(&self) -> Result<(), Box<dyn Error>> {
        match self.process_id {
            SYSTEM_IDLE_PROCESS_PROCESS_ID => Err(format!(
                "Not allowed to kill System Idle Process. process id: {}",
                self.process_id
            )
            .into()),
            SYSTEM_PROCESS_ID => Err(format!(
                "Not allowed to kill System. process id: {}",
                self.process_id
            )
            .into()),
            _ => Ok(()),
        }
    }

    fn get_process_infos(&self) -> Result<Vec<ProcessInfo>, Box<dyn Error>> {
        let mut process_infos = Vec::new();
        let mut error = None;
        unsafe {
            let snapshot_handle = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0)?;
            {
                // do NOT return early from this block
                let mut process_entry = std::mem::zeroed::<PROCESSENTRY32>();
                process_entry.dwSize = std::mem::size_of::<PROCESSENTRY32>() as u32;
                match Process32First(snapshot_handle, &mut process_entry) {
                    Ok(_) => loop {
                        let exe_file = ffi::CStr::from_ptr(process_entry.szExeFile.as_ptr() as _)
                            .to_string_lossy()
                            .into_owned();
                        println!(
                            "process id: {}, parent process id: {}, exe file: {}",
                            process_entry.th32ProcessID,
                            process_entry.th32ParentProcessID,
                            exe_file
                        );
                        process_infos.push(ProcessInfo {
                            process_id: process_entry.th32ProcessID,
                            parent_process_id: process_entry.th32ParentProcessID,
                        });
                        match Process32Next(snapshot_handle, &mut process_entry) {
                            Ok(_) => {}
                            Err(e) => {
                                if e.code() != ERROR_NO_MORE_FILES.into() {
                                    error = Some(e);
                                }
                                break;
                            }
                        }
                    },
                    Err(e) => {
                        error = Some(e);
                    }
                }
            }
            CloseHandle(snapshot_handle)?;
        }
        if let Some(e) = error {
            Err(e.into())
        } else {
            Ok(process_infos)
        }
    }

    fn terminate_process(&self, process_id: u32) -> Result<(), Box<dyn Error>> {
        let result;
        unsafe {
            let process_handle = OpenProcess(PROCESS_TERMINATE, false, process_id)?;
            {
                // do NOT return early from this block
                result = TerminateProcess(process_handle, 1).or_else(|e| {
                    if e.code() == E_ACCESSDENIED.into() {
                        // Access is denied.
                        // This happens when the process is already terminated.
                        // This is not an error.
                        Ok(())
                    } else {
                        Err(e.into())
                    }
                });
            }
            CloseHandle(process_handle)?;
        }
        result
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
            "Not allowed to kill System Idle Process. process id: 0"
        );
    }

    #[test]
    fn process_id_4() {
        let result = kill_tree_with_config(4, Config::default());
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Not allowed to kill System. process id: 4"
        );
    }
}
