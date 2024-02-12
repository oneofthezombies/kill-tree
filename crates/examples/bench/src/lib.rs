use std::{process::Command, sync::mpsc, thread};
use tracing::{subscriber, Level};
use tracing_subscriber::FmtSubscriber;

/// Initialize the global logger.
///
/// # Panics
/// If setting the default subscriber fails.
pub fn init_log() {
    subscriber::set_global_default(
        FmtSubscriber::builder()
            .with_max_level(Level::TRACE)
            .finish(),
    )
    .expect("setting default subscriber failed");
}

pub struct Unit {
    pub handle: thread::JoinHandle<()>,
    pub target_process_id: u32,
}

pub struct Output {
    pub count: u32,
    pub total_ms: u128,
    pub average_ms: u128,
}

pub fn run(count: u32, command_builder: fn(u32) -> Command) -> Output {
    let mut units = Vec::with_capacity(count as usize);
    for _ in 0..count {
        let (tx, rx) = mpsc::channel();
        let handle = thread::spawn(move || {
            let mut child = Command::new("node")
                .arg("-e")
                .arg("setInterval(() => {}, 1000)")
                .spawn()
                .unwrap();
            let target_process_id = child.id();
            tx.send(target_process_id).unwrap();
            let _ = child.wait();
        });
        let target_process_id = rx.recv().unwrap();
        units.push(Unit {
            handle,
            target_process_id,
        });
    }
    let start = std::time::Instant::now();
    for unit in &units {
        let output = command_builder(unit.target_process_id)
            .output()
            .expect("failed to execute process");
        assert!(output.status.success());
    }
    let total = start.elapsed();
    let average = total / count;
    for unit in units {
        let _ = unit.handle.join();
    }
    Output {
        count,
        total_ms: total.as_millis(),
        average_ms: average.as_millis(),
    }
}
