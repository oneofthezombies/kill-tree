use std::error::Error;

pub struct Config {
    signal: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            signal: "SIGTERM".to_string(),
        }
    }
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
}
