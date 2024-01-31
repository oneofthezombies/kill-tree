#[cfg(target_os = "linux")]
fn get_max_pid() -> u32 {
    0x400000
}

fn validate_pid(pid: u32) -> Result<(), Box<dyn std::error::Error>> {
    let max_pid = get_max_pid();
    if pid <= max_pid {
        Ok(())
    } else {
        Err(format!(
            "pid is greater than max pid. pid: {}, max pid: {}",
            pid, max_pid
        )
        .into())
    }
}

#[cfg(target_family = "unix")]
pub fn kill_tree(pid: u32) -> Result<(), Box<dyn std::error::Error>> {
    unix_impl::kill_tree(pid)
}

#[cfg(target_family = "unix")]
mod unix_impl {
    use super::*;
    use nix::{
        sys::signal::{kill, Signal},
        unistd::Pid,
    };
    use std::collections::VecDeque;

    fn parse_pid(pid: u32) -> Result<Pid, Box<dyn std::error::Error>> {
        if pid <= 0x400000 {
            Ok(Pid::from_raw(pid as i32))
        } else {
            Err(format!("Unacceptable pid value: {}", pid).into())
        }
    }

    pub(crate) fn kill_tree(pid: u32) -> Result<(), Box<dyn std::error::Error>> {
        let parsed_pid = parse_pid(pid)?;
        Ok(())
    }
}

pub(crate) fn kill_tree_with_config(pid: u32, config: &Config) -> Result<(), Box<dyn Error>> {
    assert!(false, "Not implemented");
    Ok(())
}
