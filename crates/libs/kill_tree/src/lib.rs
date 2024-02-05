use common::Impl;
pub use common::{tree, ProcessId};

mod common;
#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(unix)]
mod unix;
#[cfg(windows)]
mod windows;

/// Kills all of target process and its children recursively with the given signal.
/// Signal value is ignored on Windows.
pub async fn kill_tree_with_signal(
    process_id: ProcessId,
    signal: &str,
) -> common::Result<tree::Outputs> {
    Impl {
        process_id,
        signal: signal.to_owned(),
    }
    .kill_tree()
    .await
}

/// Kills all of target process and its children recursively with SIGTERM signal.
pub async fn kill_tree(process_id: ProcessId) -> common::Result<tree::Outputs> {
    kill_tree_with_signal(process_id, "SIGTERM").await
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
