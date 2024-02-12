use example_bench::{init_log, run};
use std::process::Command;

static EXE_PATH: &str = "taskkill.exe";

fn main() {
    init_log();
    let output = run(200, |target_process_id| {
        let mut command = Command::new(EXE_PATH);
        command.arg("/F").arg("/T").arg("/PID");
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
