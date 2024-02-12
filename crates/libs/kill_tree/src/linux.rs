use crate::{
    core::{Error, KillableBuildable, ProcessId, ProcessInfo, ProcessInfos, Result},
    unix::Killer,
    Config,
};
use tracing::{debug, instrument};

/// In hexadecimal, 0x400000.  
/// In decimal, 4194304.  
pub(crate) const AVAILABLE_MAX_PROCESS_ID: u32 = 0x0040_0000;

pub(crate) fn validate_process_id(process_id: ProcessId) -> Result<()> {
    crate::unix::validate_process_id(process_id, AVAILABLE_MAX_PROCESS_ID)
}

fn parse_status(process_id: ProcessId, status_path: String, status: &str) -> Result<ProcessInfo> {
    let mut parent_process_id = None;
    let mut name = None;
    for line in status.lines() {
        if parent_process_id.is_some() && name.is_some() {
            break;
        }

        if line.starts_with("PPid:") {
            let Some(parent_process_id_str) = line.split_whitespace().nth(1) else {
                return Err(Error::InvalidProcEntry {
                    process_id,
                    path: status_path,
                    reason: "PPid line is invalid".into(),
                    source: None,
                });
            };

            let parent_process_id_value = match parent_process_id_str.parse::<u32>() {
                Ok(x) => x,
                Err(e) => {
                    return Err(Error::InvalidProcEntry {
                        process_id,
                        path: status_path,
                        reason: "Failed to parse parent process id".into(),
                        source: Some(e),
                    });
                }
            };

            parent_process_id = Some(parent_process_id_value);
        }

        if line.starts_with("Name:") {
            let name_value = if let Some(x) = line.split_whitespace().nth(1) {
                x.to_string()
            } else {
                return Err(Error::InvalidProcEntry {
                    process_id,
                    path: status_path,
                    reason: "Name line is invalid".into(),
                    source: None,
                });
            };

            name = Some(name_value);
        }
    }

    let Some(parent_process_id) = parent_process_id else {
        return Err(Error::InvalidProcEntry {
            process_id,
            path: status_path,
            reason: "Parent process id is None".into(),
            source: None,
        });
    };

    let Some(name) = name else {
        return Err(Error::InvalidProcEntry {
            process_id,
            path: status_path,
            reason: "Name is None".into(),
            source: None,
        });
    };

    Ok(ProcessInfo {
        process_id,
        parent_process_id,
        name,
    })
}

fn parse_proc_entry(process_id: ProcessId, path: &std::path::Path) -> Result<std::path::PathBuf> {
    if !path.is_dir() {
        return Err(Error::InvalidProcEntry {
            process_id,
            path: path.display().to_string(),
            reason: "Proc entry is not a directory".into(),
            source: None,
        });
    }

    let Some(file_name) = path.file_name().and_then(|s| s.to_str()) else {
        return Err(Error::InvalidProcEntry {
            process_id,
            path: path.display().to_string(),
            reason: "Failed to get file name".into(),
            source: None,
        });
    };

    let process_id = match file_name.parse::<u32>() {
        Ok(x) => x,
        Err(e) => {
            return Err(Error::InvalidProcEntry {
                process_id,
                path: path.display().to_string(),
                reason: "Failed to parse process id".into(),
                source: Some(e),
            });
        }
    };

    let status_path = path.join("status");
    if !status_path.exists() {
        return Err(Error::InvalidProcEntry {
            process_id,
            path: status_path.display().to_string(),
            reason: "Status path does not exist".into(),
            source: None,
        });
    }

    if !status_path.is_file() {
        return Err(Error::InvalidProcEntry {
            process_id,
            path: status_path.display().to_string(),
            reason: "Status path is not a file".into(),
            source: None,
        });
    }

    Ok(status_path)
}

pub(crate) fn child_process_id_map_filter(_process_info: &ProcessInfo) -> bool {
    false
}

pub(crate) struct KillerBuilder {}

impl KillableBuildable for KillerBuilder {
    fn new_killable(&self, config: &Config) -> Result<Killer> {
        let killer_builder = crate::unix::KillerBuilder {};
        killer_builder.new_killable(config)
    }
}

#[cfg(feature = "blocking")]
pub(crate) mod blocking {
    use super::{
        debug, instrument, parse_proc_entry, parse_status, ProcessId, ProcessInfo, ProcessInfos,
        Result,
    };
    use crate::core::blocking::ProcessInfosProvidable;

    #[instrument]
    pub(crate) fn get_process_info(
        process_id: ProcessId,
        path: std::path::PathBuf,
    ) -> Result<ProcessInfo> {
        let status_path = parse_proc_entry(process_id, &path)?;
        let status = match std::fs::read_to_string(&status_path) {
            Ok(x) => x,
            Err(e) => {
                return Err(e.into());
            }
        };
        parse_status(process_id, status_path.display().to_string(), &status)
    }

    #[instrument]
    pub(crate) fn get_process_infos() -> Result<ProcessInfos> {
        let read_dir = std::fs::read_dir("/proc")?;
        let mut process_infos = ProcessInfos::new();
        for entry_result in read_dir {
            let entry = entry_result?;
            let file_name = entry.file_name();
            let Some(file_name) = file_name.to_str() else {
                debug!(file_name = ?file_name, "Failed to convert file name to str");
                continue;
            };
            let process_id = match file_name.parse::<u32>() {
                Ok(x) => x,
                Err(e) => {
                    debug!(file_name, error = ?e, "Failed to parse process id");
                    continue;
                }
            };
            let process_info = match get_process_info(process_id, entry.path()) {
                Ok(x) => x,
                Err(e) => {
                    debug!(error = ?e, "Failed to get process info");
                    continue;
                }
            };
            process_infos.push(process_info);
        }
        Ok(process_infos)
    }

    pub(crate) struct ProcessInfosProvider {}

    impl ProcessInfosProvidable for ProcessInfosProvider {
        fn get_process_infos(&self) -> Result<ProcessInfos> {
            crate::linux::blocking::get_process_infos()
        }
    }
}

#[cfg(feature = "tokio")]
pub(crate) mod tokio {
    use super::{
        debug, instrument, parse_proc_entry, parse_status, Error, ProcessId, ProcessInfo,
        ProcessInfos, Result,
    };
    use crate::core::tokio::ProcessInfosProvidable;

    #[instrument]
    pub(crate) async fn get_process_info(
        process_id: ProcessId,
        path: std::path::PathBuf,
    ) -> Result<ProcessInfo> {
        let status_path = parse_proc_entry(process_id, &path)?;
        let status = match ::tokio::fs::read_to_string(&status_path).await {
            Ok(x) => x,
            Err(e) => {
                return Err(Error::Io(e));
            }
        };
        parse_status(process_id, status_path.display().to_string(), &status)
    }

    #[instrument]
    pub(crate) async fn get_process_infos() -> Result<ProcessInfos> {
        let mut read_dir = ::tokio::fs::read_dir("/proc").await?;
        let mut process_infos = ProcessInfos::new();
        while let Some(entry) = read_dir.next_entry().await? {
            let file_name = entry.file_name();
            let Some(file_name) = file_name.to_str() else {
                debug!(file_name = ?file_name, "Failed to convert file name to str");
                continue;
            };
            let process_id = match file_name.parse::<u32>() {
                Ok(x) => x,
                Err(e) => {
                    debug!(file_name, error = ?e, "Failed to parse process id");
                    continue;
                }
            };
            let process_info = match get_process_info(process_id, entry.path()).await {
                Ok(x) => x,
                Err(e) => {
                    debug!(error = ?e, "Failed to get process info");
                    continue;
                }
            };
            process_infos.push(process_info);
        }
        Ok(process_infos)
    }

    pub(crate) struct ProcessInfosProvider {}

    impl ProcessInfosProvidable for ProcessInfosProvider {
        async fn get_process_infos(&self) -> Result<ProcessInfos> {
            crate::linux::tokio::get_process_infos().await
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_process_id_kernel_process_id() {
        let process_id = 0;
        let result = validate_process_id(process_id);
        assert!(result.is_err());
    }

    #[test]
    fn validate_process_id_init_process_id() {
        let process_id = 1;
        let result = validate_process_id(process_id);
        assert!(result.is_err());
    }

    #[test]
    fn validate_process_id_available_max_process_id() {
        let process_id = AVAILABLE_MAX_PROCESS_ID;
        validate_process_id(process_id).expect("Failed to validate process id");
    }

    #[test]
    fn parse_status_kernel_process_id() {
        let process_id = 0;
        let status_path = "/proc/0/status".to_string();
        let status = "Name: init\nPPid: 0\n".to_string();
        let process_info =
            parse_status(process_id, status_path, &status).expect("Failed to parse status");
        assert_eq!(process_info.process_id, process_id);
        assert_eq!(process_info.parent_process_id, 0);
        assert_eq!(process_info.name, "init");
    }

    #[test]
    fn parse_proc_entry_not_dir() {
        let process_id = 0;
        let path = std::path::PathBuf::from("/proc/0/status");
        let result = parse_proc_entry(process_id, &path);
        assert!(result.is_err());
    }

    #[test]
    fn parse_proc_entry_failed_to_get_file_name() {
        let process_id = 0;
        let path = std::path::PathBuf::from("/proc/");
        let result = parse_proc_entry(process_id, &path);
        assert!(result.is_err());
    }

    #[test]
    fn parse_proc_entry_failed_to_parse_process_id() {
        let process_id = 0;
        let path = std::path::PathBuf::from("/proc/invalid");
        let result = parse_proc_entry(process_id, &path);
        assert!(result.is_err());
    }

    #[test]
    fn parse_proc_entry_status_path_does_not_exist() {
        let process_id = 0;
        let path = std::path::PathBuf::from("/proc/0");
        let result = parse_proc_entry(process_id, &path);
        assert!(result.is_err());
    }

    #[test]
    fn parse_proc_entry_status_path_is_not_file() {
        let process_id = 0;
        let path = std::path::PathBuf::from("/proc/0");
        let result = parse_proc_entry(process_id, &path);
        assert!(result.is_err());
    }

    #[test]
    fn parse_proc_entry_1() {
        let process_id = 1;
        let path = std::path::PathBuf::from("/proc/1");
        let status_path = parse_proc_entry(process_id, &path).expect("Failed to parse proc entry");
        assert_eq!(status_path, std::path::PathBuf::from("/proc/1/status"));
    }

    #[test]
    fn parse_proc_entry_1_and_status() {
        let process_id = 1;
        let path = std::path::PathBuf::from("/proc/1");
        let status_path = parse_proc_entry(process_id, &path).expect("Failed to parse proc entry");
        let status = match std::fs::read_to_string(&status_path) {
            Ok(x) => x,
            Err(e) => {
                panic!("Failed to read status. error: {e}");
            }
        };
        let process_info = parse_status(process_id, status_path.display().to_string(), &status)
            .expect("Failed to parse status");
        assert_eq!(process_info.process_id, process_id);
        assert_eq!(process_info.parent_process_id, 0);
        assert_eq!(process_info.name, "systemd");
    }

    #[test]
    fn child_process_id_map_filter_false() {
        let process_info = ProcessInfo {
            process_id: 0,
            parent_process_id: 0,
            name: "init".to_string(),
        };
        assert!(!child_process_id_map_filter(&process_info));
    }

    #[cfg(feature = "blocking")]
    #[test]
    fn get_process_info_blocking() {
        let process_id = 1;
        let path = std::path::PathBuf::from("/proc/1");
        let process_info =
            blocking::get_process_info(process_id, path).expect("Failed to get process info");
        assert_eq!(process_info.process_id, process_id);
        assert_eq!(process_info.parent_process_id, 0);
        assert_eq!(process_info.name, "systemd");
    }

    #[cfg(feature = "blocking")]
    #[test]
    fn get_process_infos_blocking() {
        let process_infos = blocking::get_process_infos().expect("Failed to get process infos");
        assert!(process_infos.len() > 1);
    }

    #[cfg(feature = "tokio")]
    #[test]
    fn get_process_info_tokio() {
        let rt = ::tokio::runtime::Runtime::new().unwrap();
        let process_id = 1;
        let path = std::path::PathBuf::from("/proc/1");
        let process_info = rt
            .block_on(tokio::get_process_info(process_id, path))
            .expect("Failed to get process info");
        assert_eq!(process_info.process_id, process_id);
        assert_eq!(process_info.parent_process_id, 0);
        assert_eq!(process_info.name, "systemd");
    }

    #[cfg(feature = "tokio")]
    #[test]
    fn get_process_infos_tokio() {
        let rt = ::tokio::runtime::Runtime::new().unwrap();
        let process_infos = rt
            .block_on(tokio::get_process_infos())
            .expect("Failed to get process infos");
        assert!(process_infos.len() > 1);
    }
}
