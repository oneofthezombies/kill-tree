use std::collections::{HashMap, VecDeque};

use tokio::task::JoinSet;
use tracing::debug;

pub(crate) type Error = Box<dyn std::error::Error + Send + Sync>;
pub(crate) type Result<T> = std::result::Result<T, Error>;

pub type ProcessId = u32;
pub type ParentProcessId = ProcessId;
pub(crate) type ChildProcessId = ProcessId;
pub(crate) type ChildProcessIds = Vec<ChildProcessId>;
pub(crate) type ProcessIds = Vec<ProcessId>;

#[derive(Debug)]
pub(crate) struct ProcessInfo {
    pub(crate) process_id: ProcessId,
    pub(crate) parent_process_id: ParentProcessId,
    pub(crate) name: String,
}

pub(crate) type ProcessInfos = Vec<ProcessInfo>;
pub(crate) type ChildProcessIdMap = HashMap<ProcessId, ChildProcessIds>;
pub(crate) type ProcessInfoMap = HashMap<ProcessId, ProcessInfo>;

pub(crate) mod single {
    use super::*;

    pub(crate) enum Output {
        Killed {
            process_id: ProcessId,
        },
        MaybeAlreadyTerminated {
            process_id: ProcessId,
            reason: Error,
        },
    }
}

pub mod tree {
    use super::*;

    #[derive(Debug)]
    pub enum Output {
        Killed {
            process_id: ProcessId,
            parent_process_id: ParentProcessId,
            name: String,
        },
        MaybeAlreadyTerminated {
            process_id: ProcessId,
            reason: Error,
        },
    }

    pub type Outputs = Vec<Output>;
}

pub(crate) struct Impl {
    pub(crate) process_id: ProcessId,
    pub(crate) signal: String,
}

impl Impl {
    pub(crate) async fn kill_tree_impl(
        &self,
        filter: impl Fn(&ProcessInfo) -> bool,
        kill: impl Fn(ProcessId) -> Result<single::Output> + Send + Copy + 'static,
    ) -> Result<tree::Outputs> {
        self.validate_process_id()?;
        let process_infos = self.get_process_infos().await?;
        let child_process_id_map = get_child_process_id_map(&process_infos, filter);
        let process_ids_to_kill = get_process_ids_to_kill(self.process_id, &child_process_id_map);
        let mut tasks: JoinSet<Option<single::Output>> = JoinSet::new();
        // traverse in reverse order to kill children first
        for &process_id in process_ids_to_kill.iter().rev() {
            tasks.spawn(async move {
                let output = match kill(process_id) {
                    Ok(x) => x,
                    Err(e) => {
                        debug!(error = ?e, "failed to kill process");
                        return None;
                    }
                };
                Some(output)
            });
        }

        let mut process_info_map = get_process_info_map(process_infos);
        let mut outputs = Vec::new();
        while let Some(result) = tasks.join_next().await {
            match result {
                Ok(Some(single::Output::Killed { process_id })) => {
                    let process_info = match process_info_map.remove(&process_id) {
                        Some(x) => x,
                        None => {
                            debug!(process_id, "process info not found");
                            continue;
                        }
                    };

                    outputs.push(tree::Output::Killed {
                        process_id: process_info.process_id,
                        parent_process_id: process_info.parent_process_id,
                        name: process_info.name,
                    });
                }
                Ok(Some(single::Output::MaybeAlreadyTerminated { process_id, reason })) => {
                    outputs.push(tree::Output::MaybeAlreadyTerminated { process_id, reason });
                }
                Ok(None) => {}
                Err(e) => {
                    debug!(error = ?e, "failed to join task");
                    continue;
                }
            }
        }
        Ok(outputs)
    }
}

fn get_child_process_id_map(
    infos: &[ProcessInfo],

    // filter to map if true
    filter: impl Fn(&ProcessInfo) -> bool,
) -> ChildProcessIdMap {
    let mut map = ChildProcessIdMap::new();
    for info in infos {
        if filter(info) {
            continue;
        }
        let children = map.entry(info.parent_process_id).or_default();
        children.push(info.process_id);
    }
    for (_, children) in map.iter_mut() {
        children.sort_unstable();
    }
    map
}

fn get_process_info_map(infos: ProcessInfos) -> ProcessInfoMap {
    let mut map = ProcessInfoMap::new();
    for info in infos {
        map.insert(info.process_id, info);
    }
    map
}

/// target process id is first and children are next and grandchildren are next and so on
fn get_process_ids_to_kill(
    process_id: ProcessId,
    child_process_id_map: &ChildProcessIdMap,
) -> ProcessIds {
    let mut process_ids_to_kill = Vec::new();
    let mut queue = VecDeque::new();
    queue.push_back(process_id);
    while let Some(process_id) = queue.pop_front() {
        process_ids_to_kill.push(process_id);
        if let Some(children) = child_process_id_map.get(&process_id) {
            for &child in children {
                queue.push_back(child);
            }
        }
    }
    process_ids_to_kill
}
