// crates/examples/readme/src/kill_tree_tokio.rs
use kill_tree::{get_available_max_process_id, tokio::kill_tree, Result};

#[tokio::main]
async fn main() -> Result<()> {
    let outputs = kill_tree(get_available_max_process_id()).await?;
    println!("outputs: {outputs:?}"); // The `outputs` value is the same as the example `kill_tree`.
    Ok(())
}
