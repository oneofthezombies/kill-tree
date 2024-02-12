// crates/examples/readme/src/kill_tree.rs
use kill_tree::{blocking::kill_tree, Output, Result};

fn main() -> Result<()> {
    let process_id = 777;
    let outputs = kill_tree(process_id)?;
    for (index, output) in outputs.iter().enumerate() {
        match output {
            Output::Killed {
                process_id,
                parent_process_id,
                name,
            } => {
                println!(
                    "[{index}] Killed process. process id: {process_id}, parent process id: {parent_process_id}, name: {name}"
                );
            }
            Output::MaybeAlreadyTerminated { process_id, source } => {
                println!(
                    "[{index}] Maybe already terminated process. process id: {process_id}, source: {source}"
                );
            }
        }
    }
    Ok(())
}
