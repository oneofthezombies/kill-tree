use crate::core::{
    Config, Error, KillOutput, Killable, KillableBuildable, ProcessId, ProcessInfo, ProcessInfos,
    Result,
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

/// In hexadecimal, 0xFFFFFFFF.  
/// In decimal, 4294967295.  
/// But actually process IDs are generated as multiples of 4.  
pub(crate) const AVAILABLE_MAX_PROCESS_ID: u32 = u32::MAX;

pub(crate) fn validate_process_id(process_id: ProcessId) -> Result<()> {
    match process_id {
        SYSTEM_IDLE_PROCESS_PROCESS_ID => Err(Error::InvalidProcessId {
            process_id,
            reason: "Not allowed to kill System Idle Process".into(),
        }),
        SYSTEM_PROCESS_ID => Err(Error::InvalidProcessId {
            process_id,
            reason: "Not allowed to kill System".into(),
        }),
        _ => Ok(()),
    }
}

pub(crate) fn child_process_id_map_filter(process_info: &ProcessInfo) -> bool {
    // this process is System Idle Process
    process_info.parent_process_id == process_info.process_id
}

pub(crate) struct Killer {}

impl Killable for Killer {
    fn kill(&self, process_id: ProcessId) -> Result<KillOutput> {
        crate::windows::kill(process_id)
    }
}

pub(crate) struct KillerBuilder {}

impl KillableBuildable for KillerBuilder {
    fn new_killable(&self, _config: &Config) -> Result<Killer> {
        Ok(Killer {})
    }
}

#[instrument]
pub(crate) fn kill(process_id: ProcessId) -> Result<KillOutput> {
    let result: Result<KillOutput>;
    unsafe {
        let open_result = OpenProcess(PROCESS_TERMINATE, false, process_id);
        match open_result {
            Ok(process_handle) => {
                {
                    // do NOT return early from this block
                    result = TerminateProcess(process_handle, 1)
                        .and(Ok(KillOutput::Killed { process_id }))
                        .or_else(|e| {
                            if e.code() == E_ACCESSDENIED {
                                // Access is denied.
                                // This happens when the process is already terminated.
                                // This treat as success.
                                Ok(KillOutput::MaybeAlreadyTerminated {
                                    process_id,
                                    source: e.into(),
                                })
                            } else {
                                Err(e.into())
                            }
                        });
                }
                CloseHandle(process_handle)?;
            }
            Err(e) => {
                if e.code() == E_INVALIDARG {
                    // The parameter is incorrect.
                    // This happens when the process is already terminated.
                    // This treat as success.
                    result = Ok(KillOutput::MaybeAlreadyTerminated {
                        process_id,
                        source: e.into(),
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
pub(crate) fn get_process_infos() -> Result<ProcessInfos> {
    let mut process_infos = ProcessInfos::new();
    let mut error: Option<Error> = None;
    unsafe {
        let snapshot_handle = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0)?;
        {
            // do NOT return early from this block
            let mut process_entry = std::mem::zeroed::<PROCESSENTRY32>();
            let process_entry_size = u32::try_from(std::mem::size_of::<PROCESSENTRY32>());
            match process_entry_size {
                Ok(process_entry_size) => {
                    process_entry.dwSize = process_entry_size;
                    match Process32First(snapshot_handle, &mut process_entry) {
                        Ok(()) => loop {
                            process_infos.push(ProcessInfo {
                                process_id: process_entry.th32ProcessID,
                                parent_process_id: process_entry.th32ParentProcessID,
                                name: ffi::CStr::from_ptr(process_entry.szExeFile.as_ptr().cast())
                                    .to_string_lossy()
                                    .into_owned(),
                            });
                            match Process32Next(snapshot_handle, &mut process_entry) {
                                Ok(()) => {}
                                Err(e) => {
                                    if e.code() != ERROR_NO_MORE_FILES.into() {
                                        error = Some(e.into());
                                    }
                                    break;
                                }
                            }
                        },
                        Err(e) => {
                            error = Some(e.into());
                        }
                    }
                }
                Err(e) => {
                    error = Some(Error::InvalidCast {
                        source: e,
                        reason: "size of PROCESSENTRY32 to u32".into(),
                    });
                }
            }
        }
        CloseHandle(snapshot_handle)?;
    }
    if let Some(e) = error {
        Err(e)
    } else {
        Ok(process_infos)
    }
}

#[cfg(feature = "blocking")]
pub(crate) mod blocking {
    use super::{ProcessInfos, Result};
    use crate::core::blocking::ProcessInfosProvidable;

    pub(crate) struct ProcessInfosProvider {}

    impl ProcessInfosProvidable for ProcessInfosProvider {
        fn get_process_infos(&self) -> Result<ProcessInfos> {
            crate::windows::get_process_infos()
        }
    }
}

#[cfg(feature = "tokio")]
pub(crate) mod tokio {
    use super::{ProcessInfos, Result};
    use crate::core::tokio::ProcessInfosProvidable;

    pub(crate) struct ProcessInfosProvider {}

    impl ProcessInfosProvidable for ProcessInfosProvider {
        async fn get_process_infos(&self) -> Result<ProcessInfos> {
            crate::windows::get_process_infos()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_process_id_system_idle_process() {
        let process_id = SYSTEM_IDLE_PROCESS_PROCESS_ID;
        let result = validate_process_id(process_id);
        assert!(result.is_err());
        assert_eq!(
            format!("{}", result.unwrap_err()),
            format!(
                "Invalid process id: {process_id}. Reason: Not allowed to kill System Idle Process",
                process_id = process_id
            )
        );
    }

    #[test]
    fn validate_process_id_system() {
        let process_id = SYSTEM_PROCESS_ID;
        let result = validate_process_id(process_id);
        assert!(result.is_err());
        assert_eq!(
            format!("{}", result.unwrap_err()),
            format!(
                "Invalid process id: {process_id}. Reason: Not allowed to kill System",
                process_id = process_id
            )
        );
    }

    #[test]
    fn validate_process_id_other() {
        let process_id = 1;
        let result = validate_process_id(process_id);
        assert!(result.is_ok());
    }

    #[test]
    fn validate_process_id_available_max() {
        let process_id = AVAILABLE_MAX_PROCESS_ID;
        let result = validate_process_id(process_id);
        assert!(result.is_ok());
    }

    #[test]
    fn child_process_id_map_filter_same() {
        let process_info = ProcessInfo {
            process_id: 1,
            parent_process_id: 1,
            name: "1".to_string(),
        };
        assert_eq!(child_process_id_map_filter(&process_info), true);
    }

    #[test]
    fn child_process_id_map_filter_not_same() {
        let process_info = ProcessInfo {
            process_id: 1,
            parent_process_id: 0,
            name: "1".to_string(),
        };
        assert_eq!(child_process_id_map_filter(&process_info), false);
    }

    #[test]
    fn kill_available_max_process_id() {
        let target_process_id = AVAILABLE_MAX_PROCESS_ID;
        let result = kill(target_process_id);
        assert!(result.is_ok());
        let result = result.unwrap();
        match result {
            KillOutput::MaybeAlreadyTerminated { process_id, source } => {
                assert_eq!(target_process_id, process_id);
                match source {
                    crate::Error::Windows(e) => {
                        assert_eq!(e.code(), E_INVALIDARG);
                        assert_eq!(e.message(), "The parameter is incorrect.");
                    }
                    _ => panic!("Unexpected source: {source:?}",),
                }
            }
            _ => panic!("Unexpected result: {result:?}",),
        }
    }

    #[test]
    fn get_process_infos_test() {
        let result = get_process_infos();
        assert!(result.is_ok());
        let process_infos = result.unwrap();
        assert!(process_infos.len() > 2);
    }
}
