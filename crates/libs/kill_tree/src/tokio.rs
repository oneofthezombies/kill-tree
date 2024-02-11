use crate::{
    common::{get_child_process_id_map, get_process_ids_to_kill, get_process_info_map},
    core::{Config, KillOutput, KillOutputs, Outputs, ProcessId, Result},
    Output,
};
use tracing::debug;

#[cfg(target_os = "windows")]
use crate::windows as imp;

#[cfg(target_os = "linux")]
use crate::linux as imp;

#[cfg(target_os = "macos")]
use crate::macos as imp;

pub async fn kill_tree(process_id: ProcessId) -> Result<Outputs> {
    kill_tree_with_config(process_id, &Config::default()).await
}

pub async fn kill_tree_with_config(process_id: ProcessId, config: &Config) -> Result<Outputs> {
    imp::validate_process_id(process_id)?;
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

    #[tokio::test]
    async fn hello_world() {
        let process = Command::new("node")
            .arg("../../../tests/resources/hello_world.mjs")
            .spawn()
            .unwrap();
        let target_process_id = process.id();
        let result = kill_tree(target_process_id).await;
        assert!(result.is_ok());
        let outputs = result.unwrap();
        let output = outputs.index(0);
        if let Output::Killed {
            process_id,
            parent_process_id: _,
            name: _,
        } = output
        {
            assert_eq!(*process_id, target_process_id);
        } else {
            panic!("Unexpected output: {:?}", output);
        }
    }

    #[tokio::test]
    async fn hello_world_with_sigkill() {
        let process = Command::new("node")
            .arg("../../../tests/resources/hello_world.mjs")
            .spawn()
            .unwrap();
        let target_process_id = process.id();
        let result = kill_tree_with_signal(target_process_id, "SIGKILL").await;
        assert!(result.is_ok());
        let outputs = result.unwrap();
        let output = outputs.index(0);
        if let tree::Output::Killed {
            process_id,
            parent_process_id: _,
            name: _,
        } = output
        {
            assert_eq!(*process_id, target_process_id);
        } else {
            panic!("Unexpected output: {:?}", output);
        }
    }

    #[tokio::test]
    async fn hello_world_with_sigterm() {
        let process = Command::new("node")
            .arg("../../../tests/resources/hello_world.mjs")
            .spawn()
            .unwrap();
        let target_process_id = process.id();
        let result = kill_tree_with_signal(target_process_id, "SIGTERM").await;
        assert!(result.is_ok());
        let outputs = result.unwrap();
        let output = outputs.index(0);
        if let tree::Output::Killed {
            process_id,
            parent_process_id: _,
            name: _,
        } = output
        {
            assert_eq!(*process_id, target_process_id);
        } else {
            panic!("Unexpected output: {:?}", output);
        }
    }

    #[tokio::test]
    async fn hello_world_with_sigint() {
        let process = Command::new("node")
            .arg("../../../tests/resources/hello_world.mjs")
            .spawn()
            .unwrap();
        let target_process_id = process.id();
        let result = kill_tree_with_signal(target_process_id, "SIGINT").await;
        assert!(result.is_ok());
        let outputs = result.unwrap();
        let output = outputs.index(0);
        if let tree::Output::Killed {
            process_id,
            parent_process_id: _,
            name: _,
        } = output
        {
            assert_eq!(*process_id, target_process_id);
        } else {
            panic!("Unexpected output: {:?}", output);
        }
    }

    #[tokio::test]
    async fn hello_world_with_sigquit() {
        let process = Command::new("node")
            .arg("../../../tests/resources/hello_world.mjs")
            .spawn()
            .unwrap();
        let target_process_id = process.id();
        let result = kill_tree_with_signal(target_process_id, "SIGQUIT").await;
        assert!(result.is_ok());
        let outputs = result.unwrap();
        let output = outputs.index(0);
        if let tree::Output::Killed {
            process_id,
            parent_process_id: _,
            name: _,
        } = output
        {
            assert_eq!(*process_id, target_process_id);
        } else {
            panic!("Unexpected output: {:?}", output);
        }
    }

    #[tokio::test]
    async fn sleep() {
        let process = Command::new("node")
            .arg("../../../tests/resources/sleep.mjs")
            .spawn()
            .unwrap();
        let target_process_id = process.id();
        let result = kill_tree(target_process_id).await;
        assert!(result.is_ok());
        let outputs = result.unwrap();
        let output = outputs.index(0);
        if let tree::Output::Killed {
            process_id,
            parent_process_id: _,
            name: _,
        } = output
        {
            assert_eq!(*process_id, target_process_id);
        } else {
            panic!("Unexpected output: {:?}", output);
        }
    }

    #[tokio::test]
    async fn hello_world_wait_after() {
        let mut process = Command::new("node")
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .arg("../../../tests/resources/hello_world.mjs")
            .spawn()
            .unwrap();
        let target_process_id = process.id();
        process.wait().unwrap();
        let result = kill_tree(target_process_id).await;
        assert!(result.is_ok());
        let outputs = result.unwrap();
        let output = outputs.index(0);
        if let tree::Output::MaybeAlreadyTerminated {
            process_id,
            reason: _,
        } = output
        {
            assert_eq!(*process_id, target_process_id);
        } else {
            panic!("Unexpected output: {:?}", output);
        }
    }

    #[tokio::test]
    async fn child() {
        let process = Command::new("node")
            .arg("../../../tests/resources/child/target.mjs")
            .spawn()
            .unwrap();
        let target_process_id = process.id();
        thread::sleep(Duration::from_secs(1));
        let result = kill_tree(target_process_id).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 2);
    }

    #[tokio::test]
    async fn grandchild() {
        let process = Command::new("node")
            .arg("../../../tests/resources/grandchild/target.mjs")
            .spawn()
            .unwrap();
        let target_process_id = process.id();
        thread::sleep(Duration::from_secs(1));
        let result = kill_tree(target_process_id).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 3);
    }

    #[tokio::test]
    async fn children() {
        let process = Command::new("node")
            .arg("../../../tests/resources/children/target.mjs")
            .spawn()
            .unwrap();
        let target_process_id = process.id();
        thread::sleep(Duration::from_secs(1));
        let result = kill_tree(target_process_id).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 3);
    }

    #[tokio::test]
    async fn grandchildren() {
        let process = Command::new("node")
            .arg("../../../tests/resources/grandchildren/target.mjs")
            .spawn()
            .unwrap();
        let target_process_id = process.id();
        thread::sleep(Duration::from_secs(1));
        let result = kill_tree(target_process_id).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 5);
    }
}
