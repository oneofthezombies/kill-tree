use std::{process::Command, sync::mpsc, thread, time::Duration};
use tracing_test::traced_test;

fn get_node_script_infinite() -> String {
    r"
    // console.log('infinite. pid:', process.pid);
    setInterval(() => {}, 1000);
    "
    .to_string()
}

fn get_node_script_spawn_infinite_child() -> String {
    r#"
    // console.log('spawn child. pid:', process.pid);
    const { spawn } = require('child_process');
    // const child = spawn('node', ['-e', 'console.log("infinite. pid:", process.pid);setInterval(() => {}, 1000);'], {
    const child = spawn('node', ['-e', 'setInterval(() => {}, 1000);'], {
        stdio: 'inherit',
    });
    // child.on('exit', (code, signal) => {
    //     console.log('child process exited with ' +
    //                 `code ${code} and signal ${signal}`);
    // });
    setInterval(() => {}, 1000);
    "#
    .to_string()
}

#[traced_test]
#[test]
fn kill_tree_default() {
    let (tx, rx) = mpsc::channel();
    let thread = thread::spawn(move || {
        let mut child = Command::new("node")
            .arg("-e")
            .arg(get_node_script_infinite())
            .spawn()
            .unwrap();
        let target_process_id = child.id();
        thread::sleep(Duration::from_secs(1));
        tx.send(target_process_id).unwrap();
        let _ = child.wait();
    });
    let target_process_id = rx.recv().unwrap();
    let outputs = kill_tree::blocking::kill_tree(target_process_id).expect("Failed to kill");
    println!("{:?}", outputs);
    assert_eq!(outputs.len(), 1);
    let output = &outputs[0];
    match output {
        kill_tree::Output::Killed {
            process_id,
            parent_process_id,
            name,
        } => {
            assert_eq!(*process_id, target_process_id);
            assert_eq!(*parent_process_id, std::process::id());
            // There are cases where the process does not start with node, so a log is left for confirmation.
            println!("name: {name}");
            assert!(name.starts_with("node"));
        }
        kill_tree::Output::MaybeAlreadyTerminated { .. } => {
            panic!("This should not happen");
        }
    }
    thread.join().unwrap();
}

#[test]
fn kill_tree_with_config_sigkill() {
    let (tx, rx) = mpsc::channel();
    let thread = thread::spawn(move || {
        let mut child = Command::new("node")
            .arg("-e")
            .arg(get_node_script_infinite())
            .spawn()
            .unwrap();
        let target_process_id = child.id();
        thread::sleep(Duration::from_secs(1));
        tx.send(target_process_id).unwrap();
        let _ = child.wait();
    });
    let target_process_id = rx.recv().unwrap();
    let config = kill_tree::Config {
        signal: String::from("SIGKILL"),
        ..Default::default()
    };
    let outputs = kill_tree::blocking::kill_tree_with_config(target_process_id, &config)
        .expect("Failed to kill");
    println!("{:?}", outputs);
    assert_eq!(outputs.len(), 1);
    let output = &outputs[0];
    match output {
        kill_tree::Output::Killed {
            process_id,
            parent_process_id,
            name,
        } => {
            assert_eq!(*process_id, target_process_id);
            assert_eq!(*parent_process_id, std::process::id());
            assert!(name.starts_with("node"));
        }
        kill_tree::Output::MaybeAlreadyTerminated { .. } => {
            panic!("This should not happen");
        }
    }
    thread.join().unwrap();
}

#[traced_test]
#[test]
fn kill_tree_with_config_include_target_false() {
    let (tx, rx) = mpsc::channel();
    let thread = thread::spawn(move || {
        let mut child = Command::new("node")
            .arg("-e")
            .arg(get_node_script_spawn_infinite_child())
            .spawn()
            .unwrap();
        let target_process_id = child.id();
        thread::sleep(Duration::from_secs(1));
        tx.send(target_process_id).unwrap();
        let _ = child.wait();
    });
    let target_process_id = rx.recv().unwrap();
    let config = kill_tree::Config {
        include_target: false,
        ..Default::default()
    };
    let outputs = kill_tree::blocking::kill_tree_with_config(target_process_id, &config)
        .expect("Failed to kill");
    println!("{:?}", outputs);
    assert!(!outputs.is_empty());
    let output = &outputs[0];
    match output {
        kill_tree::Output::Killed {
            process_id: _,
            parent_process_id,
            name,
        } => {
            assert_eq!(*parent_process_id, target_process_id);
            assert!(name.starts_with("node"));
        }
        kill_tree::Output::MaybeAlreadyTerminated { .. } => {
            panic!("This should not happen");
        }
    }
    thread::sleep(Duration::from_secs(1));
    let outputs = kill_tree::blocking::kill_tree(target_process_id).expect("Failed to kill");
    println!("{:?}", outputs);
    assert_eq!(outputs.len(), 1);
    let output = &outputs[0];
    match output {
        kill_tree::Output::Killed {
            process_id,
            parent_process_id,
            name,
        } => {
            assert_eq!(*process_id, target_process_id);
            assert_eq!(*parent_process_id, std::process::id());
            assert!(name.starts_with("node"));
        }
        kill_tree::Output::MaybeAlreadyTerminated { .. } => {
            panic!("This should not happen");
        }
    }
    thread.join().unwrap();
}

#[test]
fn kill_tree_child_tree() {
    let (tx, rx) = mpsc::channel();
    let thread = thread::spawn(move || {
        let mut child = Command::new("node")
            .arg("-e")
            .arg(get_node_script_spawn_infinite_child())
            .spawn()
            .unwrap();
        thread::sleep(Duration::from_secs(1));
        let target_process_id = child.id();
        tx.send(target_process_id).unwrap();
        let _ = child.wait();
    });
    let target_process_id = rx.recv().unwrap();
    let outputs = kill_tree::blocking::kill_tree(target_process_id).expect("Failed to kill");
    assert_eq!(outputs.len(), 2);
    let target_output = outputs
        .iter()
        .find(|output| match output {
            kill_tree::Output::Killed {
                process_id: _,
                parent_process_id,
                name: _,
            } => *parent_process_id == std::process::id(),
            kill_tree::Output::MaybeAlreadyTerminated { .. } => false,
        })
        .unwrap();
    match target_output {
        kill_tree::Output::Killed {
            process_id,
            parent_process_id,
            name,
        } => {
            assert_eq!(*process_id, target_process_id);
            assert_eq!(*parent_process_id, std::process::id());
            assert!(name.starts_with("node"));
        }
        kill_tree::Output::MaybeAlreadyTerminated { .. } => {
            panic!("This should not happen");
        }
    }
    let child_output = outputs
        .iter()
        .find(|output| match output {
            kill_tree::Output::Killed {
                process_id: _,
                parent_process_id,
                name: _,
            } => *parent_process_id == target_process_id,
            kill_tree::Output::MaybeAlreadyTerminated { .. } => false,
        })
        .unwrap();
    match child_output {
        kill_tree::Output::Killed {
            process_id: _,
            parent_process_id,
            name,
        } => {
            assert_eq!(*parent_process_id, target_process_id);
            assert!(name.starts_with("node"));
        }
        kill_tree::Output::MaybeAlreadyTerminated { .. } => {
            panic!("This should not happen");
        }
    }
    thread.join().unwrap();
}
