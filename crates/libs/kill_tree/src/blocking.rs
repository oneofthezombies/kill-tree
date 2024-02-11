use crate::{
    common::{get_child_process_id_map, get_process_ids_to_kill, get_process_info_map},
    core::{Config, KillOutput, KillOutputs, Outputs, ProcessId, Result},
    Output,
};
use tracing::debug;

#[cfg(target_os = "windows")]
use crate::windows as imp;

#[cfg(target_os = "linux")]
use crate::linux as imp;

#[cfg(target_os = "macos")]
use crate::macos as imp;

pub fn kill_tree(process_id: ProcessId) -> Result<Outputs> {
    kill_tree_with_config(process_id, &Config::default())
}

pub fn kill_tree_with_config(process_id: ProcessId, config: &Config) -> Result<Outputs> {
    imp::validate_process_id(process_id)?;
    let process_infos = imp::get_process_infos()?;
    let child_process_id_map =
        get_child_process_id_map(&process_infos, imp::child_process_id_map_filter);
    let process_ids_to_kill = get_process_ids_to_kill(process_id, &child_process_id_map, config);
    // kill children first
    let mut kill_outputs = KillOutputs::new();
    for &process_id in process_ids_to_kill.iter().rev() {
        let kill_output = imp::kill(process_id, config)?;
        kill_outputs.push(kill_output);
    }
    let mut process_info_map = get_process_info_map(process_infos);
    let mut outputs = Outputs::new();
    for kill_output in kill_outputs {
        match kill_output {
            KillOutput::Killed { process_id } => {
                let Some(process_info) = process_info_map.remove(&process_id) else {
                    debug!(process_id, "process info not found");
                    continue;
                };
                outputs.push(Output::Killed {
                    process_id: process_info.process_id,
                    parent_process_id: process_info.parent_process_id,
                    name: process_info.name,
                });
            }
            KillOutput::MaybeAlreadyTerminated { process_id, source } => {
                outputs.push(Output::MaybeAlreadyTerminated { process_id, source });
            }
        }
    }
    Ok(outputs)
}
