use example_bench::{init_log, run};
use std::process::Command;

static EXE_PATH: &str = "target/release/kill_tree_cli.exe";

fn main() {
    init_log();
    let output = run(200, |target_process_id| {
        let mut command = Command::new(EXE_PATH);
        command.arg(target_process_id.to_string());
        command
    });
    println!(
        "platform: {}, arch: {}, exe: {EXE_PATH}, count: {}, total_ms: {}, average_ms: {}",
        std::env::consts::OS,
        std::env::consts::ARCH,
        output.count,
        output.total_ms,
        output.average_ms
    );
}
