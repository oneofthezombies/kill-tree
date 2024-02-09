use clap::{Parser, Subcommand};
use sheller::run;
use std::{env, io::Write, panic, path::Path};

#[derive(Parser)]
#[command(arg_required_else_help = true)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    Check,
    Clippy,
    Fmt,
    Build {
        #[arg(short, long)]
        target: String,
    },
    Test {
        #[arg(short, long)]
        target: Option<String>,
    },
    PrePush,
}

fn check() {
    run!("cargo check --workspace");
}

fn clippy() {
    run!("cargo clippy -- -D clippy::all -D clippy::pedantic");
}

fn fmt() {
    run!("cargo fmt -- --check");
}

fn build(target: &str) {
    if env::var("GITHUB_ACTIONS").is_ok() && cfg!(target_os = "linux") {
        run!("sudo apt install musl-tools");
    }

    env::set_var("RUSTFLAGS", "-C target-feature=+crt-static");
    run!("rustup target add {target}");
    run!("cargo build --package kill_tree_cli --release --target {target}");

    if env::var("GITHUB_ACTIONS").is_ok() {
        let output_path = env::var("GITHUB_OUTPUT").expect("No GITHUB_OUTPUT");
        let release_dir_path = Path::new("target").join(target).join("release");
        let windows_exe_path = release_dir_path.join("kill_tree_cli.exe");
        let file_path = if windows_exe_path.exists() {
            windows_exe_path
        } else {
            release_dir_path.join("kill_tree_cli")
        };

        if cfg!(unix) {
            run!("chmod +x {}", file_path.display());
        }

        let mut output_file = std::fs::OpenOptions::new()
            .write(true)
            .append(true)
            .open(output_path)
            .unwrap();
        writeln!(output_file, "ARTIFACT_PATH={}", file_path.display()).unwrap();
    }
}

fn test(target: Option<String>) {
    if let Some(target) = target {
        run!("cargo test --target {target}");
    } else {
        run!("cargo test --workspace");
    }
}

fn pre_push() {
    check();
    clippy();
    fmt();
    test(None);
}

fn init_log() {
    tracing::subscriber::set_global_default(
        tracing_subscriber::FmtSubscriber::builder()
            .with_max_level(tracing::Level::TRACE)
            .finish(),
    )
    .expect("setting default subscriber failed");
}

fn main() {
    init_log();
    let cli = Cli::parse();
    match cli.command {
        Some(Command::Check) => check(),
        Some(Command::Clippy) => clippy(),
        Some(Command::Fmt) => fmt(),
        Some(Command::Build { target }) => build(&target),
        Some(Command::Test { target }) => test(target),
        Some(Command::PrePush) => pre_push(),
        None => {
            panic!("No command");
        }
    }
}
