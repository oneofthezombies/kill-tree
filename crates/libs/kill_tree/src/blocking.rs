use crate::{
    common,
    core::{Config, KillOutput, KillOutputs, Outputs, ProcessId, Result},
};

#[cfg(windows)]
use crate::windows as imp;

#[cfg(target_os = "linux")]
use crate::linux as imp;

#[cfg(target_os = "macos")]
use crate::macos as imp;

pub(crate) trait Killable {
    fn kill(&self, process_id: ProcessId) -> Result<KillOutput>;
}

pub fn kill_tree(process_id: ProcessId) -> Result<Outputs> {
    kill_tree_with_config(process_id, &Config::default())
}

pub fn kill_tree_with_config(process_id: ProcessId, config: &Config) -> Result<Outputs> {
    imp::validate_process_id(process_id)?;
    let process_infos = imp::blocking::get_process_infos()?;
    let child_process_id_map =
        common::get_child_process_id_map(&process_infos, imp::child_process_id_map_filter);
    let process_ids_to_kill =
        common::get_process_ids_to_kill(process_id, &child_process_id_map, config);
    // kill children first
    let mut kill_outputs = KillOutputs::new();
    let killer = imp::blocking::new_killer(config)?;
    for &process_id in process_ids_to_kill.iter().rev() {
        let kill_output = killer.kill(process_id)?;
        kill_outputs.push(kill_output);
    }
    let mut process_info_map = common::get_process_info_map(process_infos);
    let mut outputs = Outputs::new();
    for kill_output in kill_outputs {
        let Some(output) = common::parse_kill_output(kill_output, &mut process_info_map) else {
            continue;
        };
        outputs.push(output);
    }
    Ok(outputs)
}
