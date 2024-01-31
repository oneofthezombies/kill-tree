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

pub(crate) struct ProcessInfo {
    pub(crate) process_id: u32,
    pub(crate) parent_process_id: u32,
}

pub(crate) trait TreeKillable {
    fn kill_tree(&self) -> Result<(), Box<dyn Error>>;
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
        is_to_skip: impl Fn(&ProcessInfo) -> bool,
    ) -> HashMap<u32, Vec<u32>> {
        let mut process_id_map = HashMap::new();
        for process_info in process_infos {
            let parent_process_id = process_info.parent_process_id;
            let process_id = process_info.process_id;
            if is_to_skip(process_info) {
                continue;
            }
            let children = process_id_map
                .entry(parent_process_id)
                .or_insert_with(Vec::new);
            children.push(process_id);
        }
        for (_, children) in process_id_map.iter_mut() {
            children.sort_unstable();
        }
        process_id_map
    }

    /// requested process id is first and children are next and grandchildren are next and so on
    pub(crate) fn get_process_ids_to_kill(
        &self,
        process_id_map: &HashMap<u32, Vec<u32>>,
    ) -> Vec<u32> {
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
}
