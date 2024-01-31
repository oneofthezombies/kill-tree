mod common;
#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "windows")]
mod windows;

use common::{Config, TreeKillable, TreeKiller};
use std::error::Error;

/// Kills the process and its children.
/// Returns process ids that were killed or already terminated.
pub fn kill_tree_with_config(process_id: u32, config: Config) -> Result<Vec<u32>, Box<dyn Error>> {
    TreeKiller::new(process_id, config).kill_tree()
}

/// Kills the process and its children.
/// Returns process ids that were killed or already terminated.
pub fn kill_tree(process_id: u32) -> Result<Vec<u32>, Box<dyn Error>> {
    kill_tree_with_config(process_id, Config::default())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{process::Command, thread, time::Duration};

    #[test]
    fn hello_world() {
        let process = Command::new("node")
            .arg("../tests/resources/hello_world.mjs")
            .spawn()
            .unwrap();
        let process_id = process.id();
        let result = TreeKiller::new(process_id, Config::default()).kill_tree();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), vec![process_id]);
    }

    #[test]
    fn hello_world_with_sigkill() {
        let process = Command::new("node")
            .arg("../tests/resources/hello_world.mjs")
            .spawn()
            .unwrap();
        let process_id = process.id();
        let result = TreeKiller::new(
            process_id,
            Config {
                signal: "SIGKILL".to_string(),
            },
        )
        .kill_tree();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), vec![process_id]);
    }

    #[test]
    fn hello_world_with_sigterm() {
        let process = Command::new("node")
            .arg("../tests/resources/hello_world.mjs")
            .spawn()
            .unwrap();
        let process_id = process.id();
        let result = TreeKiller::new(
            process_id,
            Config {
                signal: "SIGTERM".to_string(),
            },
        )
        .kill_tree();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), vec![process_id]);
    }

    #[test]
    fn hello_world_with_sigint() {
        let process = Command::new("node")
            .arg("../tests/resources/hello_world.mjs")
            .spawn()
            .unwrap();
        let process_id = process.id();
        let result = TreeKiller::new(
            process_id,
            Config {
                signal: "SIGINT".to_string(),
            },
        )
        .kill_tree();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), vec![process_id]);
    }

    #[test]
    fn hello_world_with_sigquit() {
        let process = Command::new("node")
            .arg("../tests/resources/hello_world.mjs")
            .spawn()
            .unwrap();
        let process_id = process.id();
        let result = TreeKiller::new(
            process_id,
            Config {
                signal: "SIGQUIT".to_string(),
            },
        )
        .kill_tree();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), vec![process_id]);
    }

    #[test]
    fn sleep() {
        let process = Command::new("node")
            .arg("../tests/resources/sleep.mjs")
            .spawn()
            .unwrap();
        let process_id = process.id();
        let result = TreeKiller::new(process_id, Config::default()).kill_tree();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), vec![process_id]);
    }

    #[test]
    fn hello_world_after_wait_1() {
        let process = Command::new("node")
            .arg("../tests/resources/hello_world.mjs")
            .spawn()
            .unwrap();
        let process_id = process.id();
        thread::sleep(Duration::from_secs(1));
        let result = TreeKiller::new(process_id, Config::default()).kill_tree();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), vec![process_id]);
    }

    #[test]
    fn child() {
        let process = Command::new("node")
            .arg("../tests/resources/child/target.mjs")
            .spawn()
            .unwrap();
        let process_id = process.id();
        thread::sleep(Duration::from_secs(1));
        let result = kill_tree_with_config(process_id, Config::default());
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 2);
    }

    #[test]
    fn grandchild() {
        let process = Command::new("node")
            .arg("../tests/resources/grandchild/target.mjs")
            .spawn()
            .unwrap();
        let process_id = process.id();
        thread::sleep(Duration::from_secs(1));
        let result = kill_tree_with_config(process_id, Config::default());
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 3);
    }

    #[test]
    fn children() {
        let process = Command::new("node")
            .arg("../tests/resources/children/target.mjs")
            .spawn()
            .unwrap();
        let process_id = process.id();
        thread::sleep(Duration::from_secs(1));
        let result = kill_tree_with_config(process_id, Config::default());
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 3);
    }

    #[test]
    fn grandchildren() {
        let process = Command::new("node")
            .arg("../tests/resources/grandchildren/target.mjs")
            .spawn()
            .unwrap();
        let process_id = process.id();
        thread::sleep(Duration::from_secs(1));
        let result = kill_tree_with_config(process_id, Config::default());
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 5);
    }
}
