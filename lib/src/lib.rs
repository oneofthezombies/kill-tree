mod common;
#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "windows")]
mod windows;

use common::{Config, TreeKillable, TreeKiller};
use std::error::Error;

pub fn kill_tree_with_config(process_id: u32, config: Config) -> Result<(), Box<dyn Error>> {
    TreeKiller::new(process_id, config).kill_tree()
}

pub fn kill_tree(process_id: u32) -> Result<(), Box<dyn Error>> {
    kill_tree_with_config(process_id, Config::default())
}
