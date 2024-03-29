use crate::core::{
    ChildProcessIdMap, ChildProcessIdMapFilter, Config, KillOutput, Killable, KillableBuildable,
    Output, Outputs, ProcessId, ProcessIds, ProcessInfo, ProcessInfoMap, ProcessInfos, Result,
};
use tracing::debug;

#[cfg(target_os = "linux")]
use crate::linux as imp;
#[cfg(target_os = "macos")]
use crate::macos as imp;
#[cfg(windows)]
use crate::windows as imp;

/// Returns the max available process ID.
/// # Platform-specifics
/// ## Windows
/// In hexadecimal, 0xFFFFFFFF.  
/// In decimal, 4294967295.  
/// But actually process IDs are generated as multiples of 4.  
///
/// ## Linux
/// In hexadecimal, 0x400000.  
/// In decimal, 4194304.  
///
/// ## Macos
/// In decimal, 99998.  
///
/// # Examples
///
/// ```
/// use kill_tree::get_available_max_process_id;
///
/// #[cfg(windows)]
/// assert!(get_available_max_process_id() == 0xFFFF_FFFF);
///
/// #[cfg(target_os = "linux")]
/// assert!(get_available_max_process_id() == 0x0040_0000);
///
/// #[cfg(target_os = "macos")]
/// assert!(get_available_max_process_id() == 99998);
/// ```
#[must_use]
pub fn get_available_max_process_id() -> u32 {
    imp::AVAILABLE_MAX_PROCESS_ID
}

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
                    "Skipping target process id"
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

pub(crate) fn parse_kill_output(
    kill_output: KillOutput,
    process_info_map: &mut ProcessInfoMap,
) -> Option<Output> {
    match kill_output {
        KillOutput::Killed { process_id } => {
            let Some(process_info) = process_info_map.remove(&process_id) else {
                debug!(process_id, "Process info not found");
                return None;
            };

            Some(Output::Killed {
                process_id: process_info.process_id,
                parent_process_id: process_info.parent_process_id,
                name: process_info.name,
            })
        }
        KillOutput::MaybeAlreadyTerminated { process_id, source } => {
            Some(Output::MaybeAlreadyTerminated { process_id, source })
        }
    }
}

pub(crate) fn kill_tree_internal(
    process_id: ProcessId,
    config: &Config,
    process_infos: ProcessInfos,
) -> Result<Outputs> {
    let child_process_id_map =
        crate::common::get_child_process_id_map(&process_infos, imp::child_process_id_map_filter);
    let process_ids_to_kill =
        crate::common::get_process_ids_to_kill(process_id, &child_process_id_map, config);
    let killable_builder = imp::KillerBuilder {};
    let killable = killable_builder.new_killable(config)?;
    let mut outputs = Outputs::new();
    let mut process_info_map = crate::common::get_process_info_map(process_infos);
    // kill children first
    for &process_id in process_ids_to_kill.iter().rev() {
        let kill_output = killable.kill(process_id)?;
        let Some(output) = crate::common::parse_kill_output(kill_output, &mut process_info_map)
        else {
            continue;
        };
        outputs.push(output);
    }
    Ok(outputs)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(windows)]
    #[test]
    fn available_max_process_id_windows() {
        assert_eq!(get_available_max_process_id(), 0xFFFF_FFFF);
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn available_max_process_id_linux() {
        assert_eq!(get_available_max_process_id(), 0x0040_0000);
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn available_max_process_id_macos() {
        assert_eq!(get_available_max_process_id(), 99998);
    }

    #[test]
    fn get_child_process_id_map_no_filter() {
        let process_infos = vec![
            ProcessInfo {
                process_id: 1,
                parent_process_id: 0,
                name: "1".to_string(),
            },
            ProcessInfo {
                process_id: 2,
                parent_process_id: 1,
                name: "2".to_string(),
            },
            ProcessInfo {
                process_id: 3,
                parent_process_id: 1,
                name: "3".to_string(),
            },
        ];
        let filter = |_: &ProcessInfo| false;
        let map = get_child_process_id_map(&process_infos, filter);
        assert_eq!(map.len(), 2);
    }

    #[test]
    fn get_child_process_id_map_filter_process_id_and_parant_process_id_is_same() {
        let process_infos = vec![
            ProcessInfo {
                process_id: 1,
                parent_process_id: 1,
                name: "1".to_string(),
            },
            ProcessInfo {
                process_id: 2,
                parent_process_id: 1,
                name: "2".to_string(),
            },
            ProcessInfo {
                process_id: 3,
                parent_process_id: 1,
                name: "3".to_string(),
            },
        ];
        let filter =
            |process_info: &ProcessInfo| process_info.process_id == process_info.parent_process_id;
        let map = get_child_process_id_map(&process_infos, filter);
        assert_eq!(map.len(), 1);
    }

    #[test]
    fn get_process_info_map_test() {
        let process_infos = vec![
            ProcessInfo {
                process_id: 1,
                parent_process_id: 0,
                name: "1".to_string(),
            },
            ProcessInfo {
                process_id: 2,
                parent_process_id: 1,
                name: "2".to_string(),
            },
            ProcessInfo {
                process_id: 3,
                parent_process_id: 1,
                name: "3".to_string(),
            },
        ];
        let map = get_process_info_map(process_infos);
        assert_eq!(map.len(), 3);
    }

    #[test]
    fn get_process_ids_to_kill_test() {
        let process_infos = vec![
            ProcessInfo {
                process_id: 1,
                parent_process_id: 0,
                name: "1".to_string(),
            },
            ProcessInfo {
                process_id: 2,
                parent_process_id: 1,
                name: "2".to_string(),
            },
            ProcessInfo {
                process_id: 3,
                parent_process_id: 1,
                name: "3".to_string(),
            },
        ];
        let child_process_id_map =
            get_child_process_id_map(&process_infos, |_: &ProcessInfo| false);
        let config = Config::default();
        let process_ids_to_kill = get_process_ids_to_kill(1, &child_process_id_map, &config);
        assert_eq!(process_ids_to_kill, vec![1, 2, 3]);
    }

    #[test]
    fn parse_kill_output_test() {
        let mut process_info_map = ProcessInfoMap::new();
        process_info_map.insert(
            1,
            ProcessInfo {
                process_id: 1,
                parent_process_id: 0,
                name: "1".to_string(),
            },
        );
        let kill_output = KillOutput::Killed { process_id: 1 };
        let output = parse_kill_output(kill_output, &mut process_info_map).expect("output is None");
        match output {
            Output::Killed {
                process_id,
                parent_process_id,
                name,
            } => {
                assert_eq!(process_id, 1);
                assert_eq!(parent_process_id, 0);
                assert_eq!(name, "1");
            }
            Output::MaybeAlreadyTerminated {
                process_id: _process_id,
                source: _source,
            } => {
                panic!("output is MaybeAlreadyTerminated");
            }
        }
    }
}
