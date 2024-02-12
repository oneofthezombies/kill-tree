// crates/examples/readme/src/kill_tree_sigkill.rs
use kill_tree::{blocking::kill_tree_with_config, Config, Result};

fn main() -> Result<()> {
    let process_id = 777;
    let config = Config {
        signal: "SIGKILL".to_string(),
        ..Default::default()
    };
    let outputs = kill_tree_with_config(process_id, &config)?;
    println!("outputs: {outputs:?}"); // The `outputs` value is the same as the example `kill_tree`.
    Ok(())
}
