mod common;
#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(unix)]
mod unix;
#[cfg(windows)]
mod windows;

pub use common::{
    Config, KillResult, KillResults, KilledInfo, MaybeAlreadyTerminatedInfo, ParentProcessId,
    ProcessId,
};
use common::{TreeKillable, TreeKiller};
use std::error::Error;

/// Kills the process and its children.
/// Returns process ids that were killed or already terminated.
pub fn kill_tree_with_config(
    process_id: u32,
    config: Config,
) -> Result<KillResults, Box<dyn Error>> {
    TreeKiller::new(process_id, config).kill_tree()
}

/// Kills the process and its children.
/// Returns process ids that were killed or already terminated.
pub fn kill_tree(process_id: u32) -> Result<KillResults, Box<dyn Error>> {
    kill_tree_with_config(process_id, Config::default())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        ops::Index,
        process::{Command, Stdio},
        thread,
        time::Duration,
    };

    #[test]
    fn hello_world() {
        println!("Hello, world!");
        let process = Command::new("node")
            .arg("../../../tests/resources/hello_world.mjs")
            .spawn()
            .unwrap();
        let process_id = process.id();
        let result = kill_tree_with_config(process_id, Config::default());
        assert!(result.is_ok());
        let kill_results = result.unwrap();
        let kill_result = kill_results.index(0);
        if let KillResult::Killed(killed_info) = kill_result {
            assert_eq!(killed_info.process_id, process_id);
        } else {
            panic!("Unexpected result: {:?}", kill_result);
        }
    }

    #[test]
    fn hello_world_with_sigkill() {
        let process = Command::new("node")
            .arg("../../../tests/resources/hello_world.mjs")
            .spawn()
            .unwrap();
        let process_id = process.id();
        let result = kill_tree_with_config(
            process_id,
            Config {
                signal: "SIGKILL".to_string(),
                ..Default::default()
            },
        );
        assert!(result.is_ok());
        let kill_results = result.unwrap();
        let kill_result = kill_results.index(0);
        if let KillResult::Killed(killed_info) = kill_result {
            assert_eq!(killed_info.process_id, process_id);
        } else {
            panic!("Unexpected result: {:?}", kill_result);
        }
    }

    #[test]
    fn hello_world_with_sigterm() {
        let process = Command::new("node")
            .arg("../../../tests/resources/hello_world.mjs")
            .spawn()
            .unwrap();
        let process_id = process.id();
        let result = kill_tree_with_config(
            process_id,
            Config {
                signal: "SIGTERM".to_string(),
                ..Default::default()
            },
        );
        assert!(result.is_ok());
        let kill_results = result.unwrap();
        let kill_result = kill_results.index(0);
        if let KillResult::Killed(killed_info) = kill_result {
            assert_eq!(killed_info.process_id, process_id);
        } else {
            panic!("Unexpected result: {:?}", kill_result);
        }
    }

    #[test]
    fn hello_world_with_sigint() {
        let process = Command::new("node")
            .arg("../../../tests/resources/hello_world.mjs")
            .spawn()
            .unwrap();
        let process_id = process.id();
        let result = kill_tree_with_config(
            process_id,
            Config {
                signal: "SIGINT".to_string(),
                ..Default::default()
            },
        );
        assert!(result.is_ok());
        let kill_results = result.unwrap();
        let kill_result = kill_results.index(0);
        if let KillResult::Killed(killed_info) = kill_result {
            assert_eq!(killed_info.process_id, process_id);
        } else {
            panic!("Unexpected result: {:?}", kill_result);
        }
    }

    #[test]
    fn hello_world_with_sigquit() {
        let process = Command::new("node")
            .arg("../../../tests/resources/hello_world.mjs")
            .spawn()
            .unwrap();
        let process_id = process.id();
        let result = kill_tree_with_config(
            process_id,
            Config {
                signal: "SIGQUIT".to_string(),
                ..Default::default()
            },
        );
        assert!(result.is_ok());
        let kill_results = result.unwrap();
        let kill_result = kill_results.index(0);
        if let KillResult::Killed(killed_info) = kill_result {
            assert_eq!(killed_info.process_id, process_id);
        } else {
            panic!("Unexpected result: {:?}", kill_result);
        }
    }

    #[test]
    fn sleep() {
        let process = Command::new("node")
            .arg("../../../tests/resources/sleep.mjs")
            .spawn()
            .unwrap();
        let process_id = process.id();
        let result = kill_tree_with_config(process_id, Config::default());
        assert!(result.is_ok());
        let kill_results = result.unwrap();
        let kill_result = kill_results.index(0);
        if let KillResult::Killed(killed_info) = kill_result {
            assert_eq!(killed_info.process_id, process_id);
        } else {
            panic!("Unexpected result: {:?}", kill_result);
        }
    }

    #[test]
    fn hello_world_wait_after() {
        let mut process = Command::new("node")
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .arg("../../../tests/resources/hello_world.mjs")
            .spawn()
            .unwrap();
        let process_id = process.id();
        process.wait().unwrap();
        let result = kill_tree_with_config(process_id, Config::default());
        assert!(result.is_ok());
        let kill_results = result.unwrap();
        let kill_result = kill_results.index(0);
        if let KillResult::MaybeAlreadyTerminated(maybe_already_terminated_info) = kill_result {
            assert_eq!(maybe_already_terminated_info.process_id, process_id);
        } else {
            panic!("Unexpected result: {:?}", kill_result);
        }
    }

    #[test]
    fn child() {
        let process = Command::new("node")
            .arg("../../../tests/resources/child/target.mjs")
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
            .arg("../../../tests/resources/grandchild/target.mjs")
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
            .arg("../../../tests/resources/children/target.mjs")
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
            .arg("../../../tests/resources/grandchildren/target.mjs")
            .spawn()
            .unwrap();
        let process_id = process.id();
        thread::sleep(Duration::from_secs(1));
        let result = kill_tree_with_config(process_id, Config::default());
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 5);
    }
}
