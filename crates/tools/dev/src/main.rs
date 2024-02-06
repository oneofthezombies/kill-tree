use clap::{Parser, Subcommand};
use std::{
    env,
    io::Write,
    panic,
    path::Path,
    process::{self, Stdio},
};

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

fn run(program: &str, args: &[&str]) {
    let mut command = process::Command::new(program);
    command
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .args(args);
    println!("Run {program} {args:?}");
    match command.status() {
        Ok(status) => {
            if !status.success() {
                eprintln!("Exit code: {:?}", status.code());
                std::process::exit(1);
            }
        }
        Err(e) => {
            eprintln!("Error: {e:?}");
            std::process::exit(1);
        }
    }
}

fn check() {
    run("cargo", &["check", "--workspace"]);
}

fn clippy() {
    run(
        "cargo",
        &[
            "clippy",
            "--",
            "-D",
            "clippy::all",
            "-D",
            "clippy::pedantic",
        ],
    );
}

fn fmt() {
    run("cargo", &["fmt", "--", "--check"]);
}

fn build(target: &str) {
    if env::var("GITHUB_ACTIONS").is_ok() && cfg!(target_os = "linux") {
        run("sudo", &["apt", "install", "musl-tools"]);
    }

    env::set_var("RUSTFLAGS", "-C target-feature=+crt-static");
    run("rustup", &["target", "add", target]);
    run(
        "cargo",
        &["build", "-p", "kill_tree_cli", "-r", "--target", target],
    );

    if env::var("GITHUB_ACTIONS").is_ok() {
        let output = env::var("GITHUB_OUTPUT").expect("No GITHUB_OUTPUT");
        let windows_path = Path::new("target")
            .join(target)
            .join("release")
            .join("kill_tree_cli.exe");
        let file_path = if windows_path.exists() {
            windows_path
        } else {
            Path::new("target")
                .join(target)
                .join("release")
                .join("kill_tree_cli")
        };

        if cfg!(unix) {
            run("chmod", &["+x", file_path.to_str().unwrap()]);
        }

        let mut output_path = std::fs::OpenOptions::new()
            .write(true)
            .append(true)
            .open(output)
            .unwrap();
        writeln!(output_path, "ARTIFACT_PATH={}", file_path.to_str().unwrap()).unwrap();
    }
}

fn test(target: Option<String>) {
    let Some(target) = target else {
        run("cargo", &["test", "--workspace"]);
        return;
    };

    run("cargo", &["test", "--target", target.as_str()]);
}

fn pre_push() {
    check();
    clippy();
    fmt();
    test(None);
}

fn main() {
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
