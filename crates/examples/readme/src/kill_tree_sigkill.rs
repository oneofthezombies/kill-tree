use kill_tree::{blocking::kill_tree_with_config, Config, Output, Result};

fn main() -> Result<()> {
    let process_id = 777;
    let config = Config {
        signal: "SIGKILL".to_string(),
        ..Default::default()
    };
    let outputs = kill_tree_with_config(process_id, &config)?;
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
