use kill_tree::{blocking::kill_tree_with_config, Config};
use std::sync::mpsc::channel;

fn cleanup_children() {
    let current_process_id = std::process::id();
    let config = Config {
        include_target: false,
        ..Default::default()
    };
    let result = kill_tree_with_config(current_process_id, &config);
    println!("kill_tree_with_config: {result:?}");
}

fn main() {
    let (tx, rx) = channel();

    ctrlc::set_handler(move || {
        cleanup_children();
        tx.send(()).expect("Could not send signal on channel.");
    })
    .expect("Error setting handler.");

    println!("Current process id: {}", std::process::id());
    println!("Waiting for signal...");
    rx.recv().expect("Could not receive from channel.");
    println!("Got it! Exiting...");
}
