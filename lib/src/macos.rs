#[cfg(target_os = "macos")]
fn get_max_pid() -> u32 {
    99998
}

pub(crate) fn kill_tree_with_config(pid: u32, config: &Config) -> Result<(), Box<dyn Error>> {
    assert!(false, "Not implemented");
    Ok(())
}
