use kill_tree::{get_available_max_process_id, tokio::kill_tree, Result};

#[tokio::main]
async fn main() -> Result<()> {
    let _ = kill_tree(get_available_max_process_id()).await?;
    Ok(())
}
