use kill_tree::kill_tree;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let process_id = 12345;
    let outputs = kill_tree(process_id).await.map_err(|e| e.to_string())?;
    for output in outputs {
        match output {
            kill_tree::tree::Output::Killed {
                process_id,
                parent_process_id,
                name,
            } => {
                println!(
                    "Killed process. process id: {}, parent process id: {}, name: {}",
                    process_id, parent_process_id, name
                );
            }
            kill_tree::tree::Output::MaybeAlreadyTerminated { process_id, reason } => {
                println!(
                    "Maybe already terminated process. process id: {}, reason: {}",
                    process_id, reason
                );
            }
        }
    }
    Ok(())
}
