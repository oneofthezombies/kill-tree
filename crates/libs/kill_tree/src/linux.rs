use crate::common::{self, Impl, ProcessInfo, ProcessInfos};
use std::path::PathBuf;
use tokio::{fs, task::JoinSet};
use tracing::{debug, instrument};

/// decimal value is 4194304
const AVAILABLE_MAX_PROCESS_ID: u32 = 0x400000;

#[instrument]
async fn get_process_info(path: PathBuf) -> Option<ProcessInfo> {
    if !path.is_dir() {
        debug!(path = ?path, "proc entry is not a directory");
        return None;
    }

    let file_name = match path.file_name().and_then(|s| s.to_str()) {
        Some(x) => x,
        None => {
            debug!(path = ?path, "failed to get file name");
            return None;
        }
    };

    let process_id = match file_name.parse::<u32>() {
        Ok(x) => x,
        Err(e) => {
            debug!(path = ?path, error = ?e, "failed to parse process id");
            return None;
        }
    };

    let status_path = path.join("status");
    if !status_path.exists() {
        debug!(path = ?status_path, "status path does not exist");
        return None;
    }

    if !status_path.is_file() {
        debug!(path = ?status_path, "status path is not a file");
        return None;
    }

    let status = match fs::read_to_string(&status_path).await {
        Ok(x) => x,
        Err(e) => {
            debug!(path = ?status_path, error = ?e, "failed to read status file");
            return None;
        }
    };

    let mut parent_process_id = None;
    let mut name = None;
    for line in status.lines() {
        if parent_process_id.is_some() && name.is_some() {
            break;
        }

        if line.starts_with("PPid:") {
            let parent_process_id_str = match line.split_whitespace().nth(1) {
                Some(x) => x,
                None => {
                    debug!(path = ?status_path, "PPid line is invalid");
                    return None;
                }
            };

            let parent_process_id_value = match parent_process_id_str.parse::<u32>() {
                Ok(x) => x,
                Err(e) => {
                    debug!(value = ?parent_process_id_str, error = ?e, "failed to parse parent process id");
                    return None;
                }
            };

            parent_process_id = Some(parent_process_id_value);
        }

        if line.starts_with("Name:") {
            let name_value = match line.split_whitespace().nth(1) {
                Some(x) => x.to_string(),
                None => {
                    debug!(path = ?status_path, "Name line is invalid");
                    return None;
                }
            };

            name = Some(name_value);
        }
    }

    let parent_process_id = match parent_process_id {
        Some(x) => x,
        None => {
            debug!(path = ?status_path, "parent process id is None");
            return None;
        }
    };

    let name = match name {
        Some(x) => x,
        None => {
            debug!(path = ?status_path, "name is None");
            return None;
        }
    };

    Some(ProcessInfo {
        process_id,
        parent_process_id,
        name,
    })
}

#[instrument]
pub(crate) async fn get_process_infos() -> common::Result<ProcessInfos> {
    let mut tasks: JoinSet<Option<ProcessInfo>> = JoinSet::new();
    let mut read_dir = fs::read_dir("/proc").await?;
    while let Some(entry) = read_dir.next_entry().await? {
        tasks.spawn(get_process_info(entry.path()));
    }
    let mut process_infos = ProcessInfos::new();
    while let Some(result) = tasks.join_next().await {
        let process_info = match result {
            Ok(x) => x,
            Err(e) => {
                debug!(error = ?e, "failed to get process info");
                continue;
            }
        };
        if let Some(process_info) = process_info {
            process_infos.push(process_info);
        }
    }
    Ok(process_infos)
}

impl Impl {
    pub(crate) fn validate_process_id(&self) -> common::Result<()> {
        crate::unix::validate_process_id(self.process_id, AVAILABLE_MAX_PROCESS_ID)
    }

    pub(crate) async fn get_process_infos(&self) -> common::Result<ProcessInfos> {
        get_process_infos().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kill_tree;

    #[tokio::test]
    async fn process_id_max_plus_1() {
        let result = kill_tree(AVAILABLE_MAX_PROCESS_ID + 1).await;
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Process id is too large. process id: 4194305, available max process id: 4194304"
        );
    }
}
