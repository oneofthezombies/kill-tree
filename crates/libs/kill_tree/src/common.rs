use tracing::debug;

use crate::{
    core::{
        ChildProcessIdMap, ChildProcessIdMapFilter, ProcessIds, ProcessInfo, ProcessInfoMap,
        ProcessInfos,
    },
    Config, ProcessId,
};

// pub(crate) struct Impl {
//     pub(crate) process_id: ProcessId,
//     pub(crate) signal: String,
// }

// impl Impl {
//     pub(crate) async fn kill_tree_impl(
//         &self,
//         filter: impl Fn(&ProcessInfo) -> bool,
//         kill: impl Fn(ProcessId) -> Result<single::Output> + Send + Copy + 'static,
//     ) -> Result<tree::Outputs> {
//         self.validate_process_id()?;
//         let process_infos = self.get_process_infos().await?;
//         let child_process_id_map = get_child_process_id_map(&process_infos, filter);
//         let process_ids_to_kill = get_process_ids_to_kill(self.process_id, &child_process_id_map);
//         let mut tasks: JoinSet<Option<single::Output>> = JoinSet::new();
//         // traverse in reverse order to kill children first
//         for &process_id in process_ids_to_kill.iter().rev() {
//             tasks.spawn(async move {
//                 let output = match kill(process_id) {
//                     Ok(x) => x,
//                     Err(e) => {
//                         debug!(error = ?e, "failed to kill process");
//                         return None;
//                     }
//                 };
//                 Some(output)
//             });
//         }

//         let mut process_info_map = get_process_info_map(process_infos);
//         let mut outputs = Vec::new();
//         while let Some(result) = tasks.join_next().await {
//             match result {
//                 Ok(Some(single::Output::Killed { process_id })) => {
//                     let Some(process_info) = process_info_map.remove(&process_id) else {
//                         debug!(process_id, "process info not found");
//                         continue;
//                     };

//                     outputs.push(tree::Output::Killed {
//                         process_id: process_info.process_id,
//                         parent_process_id: process_info.parent_process_id,
//                         name: process_info.name,
//                     });
//                 }
//                 Ok(Some(single::Output::MaybeAlreadyTerminated { process_id, reason })) => {
//                     outputs.push(tree::Output::MaybeAlreadyTerminated { process_id, reason });
//                 }
//                 Ok(None) => {}
//                 Err(e) => {
//                     debug!(error = ?e, "failed to join task");
//                     continue;
//                 }
//             }
//         }
//         Ok(outputs)
//     }
// }

/// Create a map from parent process id to child process ids.
pub(crate) fn get_child_process_id_map(
    process_infos: &[ProcessInfo],
    filter: ChildProcessIdMapFilter,
) -> ChildProcessIdMap {
    let mut map = ChildProcessIdMap::new();
    for process_info in process_infos {
        if filter(process_info) {
            continue;
        }
        let children = map.entry(process_info.parent_process_id).or_default();
        children.push(process_info.process_id);
    }
    for (_, children) in &mut map.iter_mut() {
        children.sort_unstable();
    }
    map
}

/// Create a map from process id to process info.
pub(crate) fn get_process_info_map(process_infos: ProcessInfos) -> ProcessInfoMap {
    let mut map = ProcessInfoMap::new();
    for process_info in process_infos {
        map.insert(process_info.process_id, process_info);
    }
    map
}

/// Breadth-first search to get all process ids to kill.
pub(crate) fn get_process_ids_to_kill(
    target_process_id: ProcessId,
    child_process_id_map: &ChildProcessIdMap,
    config: &Config,
) -> ProcessIds {
    let mut process_ids_to_kill = Vec::new();
    let mut queue = std::collections::VecDeque::new();
    queue.push_back(target_process_id);
    while let Some(process_id) = queue.pop_front() {
        if process_id == target_process_id {
            if config.include_target {
                process_ids_to_kill.push(process_id);
            } else {
                debug!(
                    process_id,
                    include_target = config.include_target,
                    "skipping target process id"
                );
            }
        } else {
            process_ids_to_kill.push(process_id);
        }
        if let Some(children) = child_process_id_map.get(&process_id) {
            for &child in children {
                queue.push_back(child);
            }
        }
    }
    process_ids_to_kill
}
