use crate::common::{
    KillResult, KillResults, KilledInfo, MaybeAlreadyTerminatedInfo, ProcessId, ProcessInfo,
    TreeKillable, TreeKiller,
};
use std::{error::Error, ffi};
use windows::Win32::{
    Foundation::{CloseHandle, ERROR_NO_MORE_FILES, E_ACCESSDENIED, E_INVALIDARG},
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
    fn kill_tree(&self) -> Result<KillResults, Box<dyn Error>> {
        // self.config is not used on Windows platform yet
        let _ = self.config;
        self.validate_pid()?;
        let process_infos = self.get_process_infos()?;
        let process_id_map = self.get_process_id_map(&process_infos, |process_info| {
            // this process is System Idle Process
            process_info.parent_process_id == process_info.process_id
        });
        let mut process_info_map = self.get_process_info_map(process_infos);
        let process_ids_to_kill = self.get_process_ids_to_kill(&process_id_map);
        let mut kill_results = KillResults::new();
        for &process_id in process_ids_to_kill.iter().rev() {
            let kill_result = self.kill(process_id)?;
            kill_results.push(match kill_result {
                None => {
                    if let Some(process_info) = process_info_map.remove(&process_id) {
                        KillResult::Killed(KilledInfo {
                            process_id,
                            parent_process_id: process_info.parent_process_id,
                            name: process_info.name,
                        })
                    } else {
                        KillResult::InternalError(format!(
                            "The process was killed but the process info does not exist. process id: {}",
                            process_id
                        ).into())
                    }
                }
                Some(e) => KillResult::MaybeAlreadyTerminated(MaybeAlreadyTerminatedInfo {
                    process_id: process_id,
                    reason: e,
                }),
            });
        }
        Ok(kill_results)
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
                        process_infos.push(ProcessInfo {
                            process_id: process_entry.th32ProcessID,
                            parent_process_id: process_entry.th32ParentProcessID,
                            name: ffi::CStr::from_ptr(process_entry.szExeFile.as_ptr() as _)
                                .to_string_lossy()
                                .into_owned(),
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

    fn kill(&self, process_id: ProcessId) -> Result<Option<Box<dyn Error>>, Box<dyn Error>> {
        let result;
        unsafe {
            let open_result = OpenProcess(PROCESS_TERMINATE, false, process_id);
            match open_result {
                Ok(process_handle) => {
                    {
                        // do NOT return early from this block
                        result = TerminateProcess(process_handle, 1)
                            .and(Ok(None))
                            .or_else(|e| {
                                if e.code() == E_ACCESSDENIED.into() {
                                    // Access is denied.
                                    // This happens when the process is already terminated.
                                    // This is not an error.
                                    Ok(Some(e.into()))
                                } else {
                                    Err(e.into())
                                }
                            })
                    }
                    CloseHandle(process_handle)?;
                }
                Err(e) => {
                    if e.code() == E_INVALIDARG.into() {
                        // The parameter is incorrect.
                        // This happens when the process is already terminated.
                        // This is not an error.
                        result = Ok(Some(e.into()));
                    } else {
                        result = Err(e.into());
                    }
                }
            }
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
