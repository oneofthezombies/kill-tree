use std::{
    collections::{HashMap, VecDeque},
    error::Error,
};

pub struct Config {
    /**
     * Signal to send to the process and its children.
     * Windows platform will ignore this field.
     * Default: SIGTERM
     */
    pub signal: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            signal: "SIGTERM".to_string(),
        }
    }
}

pub type ProcessId = u32;
pub type ParentProcessId = ProcessId;
pub(crate) type ProcessIds = Vec<ProcessId>;

#[derive(Debug)]
pub(crate) struct ProcessInfo {
    pub(crate) process_id: ProcessId,
    pub(crate) parent_process_id: ParentProcessId,
    pub(crate) name: String,
}

pub(crate) type ProcessInfos = Vec<ProcessInfo>;
pub(crate) type ProcessIdMap = HashMap<ProcessId, Vec<ProcessId>>;
pub(crate) type ProcessInfoMap = HashMap<ProcessId, ProcessInfo>;

#[derive(Debug)]
pub struct KilledInfo {
    pub process_id: ProcessId,
    pub parent_process_id: ParentProcessId,
    pub name: String,
}

#[derive(Debug)]
pub struct MaybeAlreadyTerminatedInfo {
    pub process_id: ProcessId,
    pub reason: Box<dyn Error>,
}

#[derive(Debug)]
pub enum KillResult {
    /// The process killed successfully.
    Killed(KilledInfo),

    /// The process does not exist.
    /// This can happen if the process was already terminated.
    MaybeAlreadyTerminated(MaybeAlreadyTerminatedInfo),

    /// It's a case that should never happen in normal situations.
    /// In an abnormal situation, I decided that it would be better for this library to return the error than to panic.
    InternalError(Box<dyn Error>),
}

pub type KillResults = Vec<KillResult>;

pub(crate) trait TreeKillable {
    /// Kills the process and its children.
    /// Returns process ids that were killed or already terminated.
    fn kill_tree(&self) -> Result<KillResults, Box<dyn Error>>;
}

pub(crate) struct TreeKiller {
    pub(crate) process_id: u32,
    pub(crate) config: Config,
}

impl TreeKiller {
    pub(crate) fn new(process_id: u32, config: Config) -> Self {
        Self { process_id, config }
    }

    pub(crate) fn get_process_id_map(
        &self,
        process_infos: &[ProcessInfo],
        filter_process_info_to_process_id_map: impl Fn(&ProcessInfo) -> bool,
    ) -> ProcessIdMap {
        let mut process_id_map = ProcessIdMap::new();
        for process_info in process_infos {
            if filter_process_info_to_process_id_map(process_info) {
                continue;
            }
            let children = process_id_map
                .entry(process_info.parent_process_id)
                .or_default();
            children.push(process_info.process_id);
        }
        for (_, children) in process_id_map.iter_mut() {
            children.sort_unstable();
        }
        process_id_map
    }

    pub(crate) fn get_process_info_map(&self, process_infos: ProcessInfos) -> ProcessInfoMap {
        let mut process_info_map = ProcessInfoMap::new();
        for process_info in process_infos {
            process_info_map.insert(process_info.process_id, process_info);
        }
        process_info_map
    }

    /// requested process id is first and children are next and grandchildren are next and so on
    pub(crate) fn get_process_ids_to_kill(&self, process_id_map: &ProcessIdMap) -> ProcessIds {
        let mut process_ids_to_kill = Vec::new();
        let mut queue = VecDeque::new();
        queue.push_back(self.process_id);
        while let Some(process_id) = queue.pop_front() {
            process_ids_to_kill.push(process_id);
            if let Some(children) = process_id_map.get(&process_id) {
                for child in children {
                    queue.push_back(*child);
                }
            }
        }
        process_ids_to_kill
    }

    pub(crate) fn kill_tree_impl(
        &self,
        filter_process_info_to_process_id_map: impl Fn(&ProcessInfo) -> bool,
        kill: impl Fn(ProcessId) -> Result<Option<Box<dyn Error>>, Box<dyn Error>>,
    ) -> Result<KillResults, Box<dyn Error>> {
        self.validate_pid()?;
        let process_infos = self.get_process_infos()?;
        let process_id_map =
            self.get_process_id_map(&process_infos, filter_process_info_to_process_id_map);
        let mut process_info_map = self.get_process_info_map(process_infos);
        let process_ids_to_kill = self.get_process_ids_to_kill(&process_id_map);
        let mut kill_results = KillResults::new();
        for &process_id in process_ids_to_kill.iter().rev() {
            let kill_result = kill(process_id)?;
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
                        process_id,
                        reason: e,
                    }),
                });
        }
        Ok(kill_results)
    }
}
