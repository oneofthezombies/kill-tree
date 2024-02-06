use crate::{
    common::{self, single, Impl, ProcessId, ProcessInfo, ProcessInfos},
    tree,
};
use std::ffi;
use tracing::instrument;
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

#[instrument]
fn kill(process_id: ProcessId) -> common::Result<single::Output> {
    let result;
    unsafe {
        let open_result = OpenProcess(PROCESS_TERMINATE, false, process_id);
        match open_result {
            Ok(process_handle) => {
                {
                    // do NOT return early from this block
                    result = TerminateProcess(process_handle, 1)
                        .and(Ok(single::Output::Killed { process_id }))
                        .or_else(|e| {
                            if e.code() == E_ACCESSDENIED {
                                // Access is denied.
                                // This happens when the process is already terminated.
                                // This treat as success.
                                Ok(single::Output::MaybeAlreadyTerminated {
                                    process_id,
                                    reason: e.into(),
                                })
                            } else {
                                Err(e.into())
                            }
                        })
                }
                CloseHandle(process_handle)?;
            }
            Err(e) => {
                if e.code() == E_INVALIDARG {
                    // The parameter is incorrect.
                    // This happens when the process is already terminated.
                    // This treat as success.
                    result = Ok(single::Output::MaybeAlreadyTerminated {
                        process_id,
                        reason: e.into(),
                    });
                } else {
                    result = Err(e.into());
                }
            }
        }
    }
    result
}

#[instrument]
pub(crate) async fn get_process_infos() -> common::Result<ProcessInfos> {
    let mut process_infos = ProcessInfos::new();
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
                        Ok(()) => {}
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

impl Impl {
    pub(crate) async fn kill_tree(&self) -> common::Result<tree::Outputs> {
        // self.signal is not used on Windows platform yet
        let _ = self.signal;
        self.kill_tree_impl(
            |process_info| {
                // this process is System Idle Process
                process_info.parent_process_id == process_info.process_id
            },
            kill,
        )
        .await
    }

    pub(crate) fn validate_process_id(&self) -> common::Result<()> {
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

    pub(crate) async fn get_process_infos(&self) -> common::Result<ProcessInfos> {
        get_process_infos().await
    }
}

#[cfg(test)]
mod tests {
    use crate::kill_tree;

    #[tokio::test]
    async fn process_id_0() {
        let result = kill_tree(0).await;
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Not allowed to kill System Idle Process. process id: 0"
        );
    }

    #[tokio::test]
    async fn process_id_4() {
        let result = kill_tree(4).await;
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Not allowed to kill System. process id: 4"
        );
    }
}
