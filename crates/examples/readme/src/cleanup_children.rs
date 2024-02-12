use kill_tree::{blocking::kill_tree_with_config, Config};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

fn cleanup_children() {
    let current_process_id = std::process::id();
    let config = Config {
        include_target: false,
        ..Default::default()
    };
    let result = kill_tree_with_config(current_process_id, &config);
    println!("kill_tree_with_config: {:?}", result);
}

fn main() {
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    ctrlc::set_handler(move || {
        cleanup_children();
        r.store(false, Ordering::SeqCst);
    })
    .expect("Error setting Ctrl-C handler");

    println!("Waiting for Ctrl-C...");
    while running.load(Ordering::SeqCst) {}
    println!("Got it! Exiting...");
}
