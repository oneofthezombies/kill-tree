use clap::{Parser, Subcommand};
use std::{
    env, panic,
    process::{Command, Stdio},
};

#[derive(Parser)]
#[command(arg_required_else_help = true)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    Check,
    Clippy,
    Fmt,
    Build { platform: String },
    Test { platform: Option<String> },
}

fn run(program: &str, args: &[&str]) {
    let mut command = Command::new(program);
    command
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .args(args);
    println!("Run {:?} {:?}", program, args);
    command
        .status()
        .expect(format!("Failed to run {:?} {:?}", program, args).as_str());
}

fn check() {
    run("cargo", &["check", "--workspace"]);
}

fn clippy() {
    run("cargo", &["clippy", "--", "-D", "clippy::pedantic"]);
}

fn fmt() {
    run("cargo", &["fmt", "--", "--check"]);
}

fn build(platform: &str) {
    env::set_var("RUSTFLAGS", "-C target-feature=+crt-static");
    if platform == "windows" {
        run("rustup", &["target", "add", "x86_64-pc-windows-msvc"]);
        run(
            "cargo",
            &[
                "build",
                "-p",
                "kill_tree_cli",
                "-r",
                "--target",
                "x86_64-pc-windows-msvc",
            ],
        );
    } else if platform == "linux" {
        run("rustup", &["target", "add", "x86_64-unknown-linux-musl"]);
        run(
            "cargo",
            &[
                "build",
                "-p",
                "kill_tree_cli",
                "-r",
                "--target",
                "x86_64-unknown-linux-musl",
            ],
        );
    } else if platform == "macos" {
        run("rustup", &["target", "add", "aarch64-apple-darwin"]);
        run("rustup", &["target", "add", "x86_64-apple-darwin"]);
        run(
            "cargo",
            &[
                "build",
                "-p",
                "kill_tree_cli",
                "-r",
                "--target",
                "aarch64-apple-darwin",
            ],
        );
        run(
            "cargo",
            &[
                "build",
                "-p",
                "kill_tree_cli",
                "-r",
                "--target",
                "x86_64-apple-darwin",
            ],
        );
    } else {
        panic!("Unsupported platform: {platform}");
    }
}

fn test(platform: Option<String>) {
    let Some(platform) = platform else {
        run("cargo", &["test", "--workspace"]);
        return;
    };

    if platform == "windows" {
        run("cargo", &["test", "--target", "x86_64-pc-windows-msvc"]);
    } else if platform == "linux" {
        run("cargo", &["test", "--target", "x86_64-unknown-linux-musl"]);
    } else if platform == "macos" {
        run("cargo", &["test", "--target", "aarch64-apple-darwin"]);
        run("cargo", &["test", "--target", "x86_64-apple-darwin"]);
    } else {
        panic!("Unsupported platform: {platform}");
    }
}

fn main() {
    panic::set_hook(Box::new(|panic_info| {
        let location = panic_info.location().unwrap();
        eprintln!(
            "Panic occurred in file '{}' at line {}",
            location.file(),
            location.line()
        );
        eprintln!("Panic message: {:?}", panic_info);
        std::process::abort();
    }));

    let cli = Cli::parse();
    match cli.command {
        Some(Commands::Check) => check(),
        Some(Commands::Clippy) => clippy(),
        Some(Commands::Fmt) => fmt(),
        Some(Commands::Build { platform }) => build(&platform),
        Some(Commands::Test { platform }) => test(platform),
        None => {
            panic!("No command");
        }
    }
}
