use crate::{
    core::{Error, Killable, ProcessId, ProcessInfo, ProcessInfos, Result},
    Config,
};
use tracing::{debug, instrument};

/// decimal value is 4194304
const AVAILABLE_MAX_PROCESS_ID: u32 = 0x0040_0000;

pub(crate) fn validate_process_id(process_id: ProcessId) -> Result<()> {
    crate::unix::validate_process_id(process_id, AVAILABLE_MAX_PROCESS_ID)
}

fn parse_status(process_id: ProcessId, status_path: String, status: String) -> Result<ProcessInfo> {
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

fn parse_proc_entry(process_id: ProcessId, path: std::path::PathBuf) -> Result<std::path::PathBuf> {
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

pub(crate) fn new_killer(config: &Config) -> Result<impl Killable> {
    crate::unix::new_killer(config)
}

#[cfg(feature = "blocking")]
pub(crate) mod blocking {
    use super::*;

    #[instrument]
    fn get_process_info(process_id: ProcessId, path: std::path::PathBuf) -> Result<ProcessInfo> {
        let status_path = parse_proc_entry(process_id, path)?;
        let status = match std::fs::read_to_string(&status_path) {
            Ok(x) => x,
            Err(e) => {
                return Err(Error::Io(e));
            }
        };
        parse_status(process_id, status_path.display().to_string(), status)
    }

    #[instrument]
    pub(crate) fn get_process_infos() -> Result<ProcessInfos> {
        let mut read_dir = std::fs::read_dir("/proc")?;
        let mut process_infos = ProcessInfos::new();
        while let Some(entry_result) = read_dir.next() {
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
}

#[cfg(feature = "tokio")]
pub(crate) mod tokio {
    use super::*;

    #[instrument]
    async fn get_process_info(
        process_id: ProcessId,
        path: std::path::PathBuf,
    ) -> Result<ProcessInfo> {
        let status_path = parse_proc_entry(process_id, path)?;
        let status = match ::tokio::fs::read_to_string(&status_path).await {
            Ok(x) => x,
            Err(e) => {
                return Err(Error::Io(e));
            }
        };
        parse_status(process_id, status_path.display().to_string(), status)
    }

    #[instrument]
    pub(crate) async fn get_process_infos() -> Result<ProcessInfos> {
        let mut tasks: ::tokio::task::JoinSet<Result<ProcessInfo>> = ::tokio::task::JoinSet::new();
        let mut read_dir = ::tokio::fs::read_dir("/proc").await?;
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
            tasks.spawn(get_process_info(process_id, entry.path()));
        }
        let mut process_infos = ProcessInfos::new();
        while let Some(join_result) = tasks.join_next().await {
            let result = join_result?;
            let process_info = match result {
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
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::kill_tree;

//     #[tokio::test]
//     async fn process_id_max_plus_1() {
//         let result = kill_tree(AVAILABLE_MAX_PROCESS_ID + 1).await;
//         assert!(result.is_err());
//         assert_eq!(
//             result.unwrap_err().to_string(),
//             "Process id is too large. process id: 4194305, available max process id: 4194304"
//         );
//     }
// }
