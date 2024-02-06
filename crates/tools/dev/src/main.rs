use clap::{Parser, Subcommand};
use std::{
    env,
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
    Build { platform: String },
    Test { platform: String },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Build { platform }) => {
            env::set_var("RUSTFLAGS", "-C target-feature=+crt-static");
            if platform == "windows" {
                Command::new("rustup")
                    .stdout(Stdio::inherit())
                    .stderr(Stdio::inherit())
                    .args(["target", "add", "x86_64-pc-windows-msvc"])
                    .status()
                    .expect("Failed to add x86_64-pc-windows-msvc target");

                Command::new("cargo")
                    .stdout(Stdio::inherit())
                    .stderr(Stdio::inherit())
                    .args([
                        "build",
                        "-p",
                        "kill_tree_cli",
                        "-r",
                        "--target",
                        "x86_64-pc-windows-msvc",
                    ])
                    .status()
                    .expect("Failed to build kill_tree_cli");
            } else if platform == "linux" {
                Command::new("rustup")
                    .stdout(Stdio::inherit())
                    .stderr(Stdio::inherit())
                    .args(["target", "add", "x86_64-unknown-linux-musl"])
                    .status()
                    .expect("Failed to add x86_64-unknown-linux-musl target");

                Command::new("cargo")
                    .stdout(Stdio::inherit())
                    .stderr(Stdio::inherit())
                    .args([
                        "build",
                        "-p",
                        "kill_tree_cli",
                        "-r",
                        "--target",
                        "x86_64-unknown-linux-musl",
                    ])
                    .status()
                    .expect("Failed to build kill_tree_cli");
            } else if platform == "macos" {
                Command::new("rustup")
                    .stdout(Stdio::inherit())
                    .stderr(Stdio::inherit())
                    .args(["target", "add", "aarch64-apple-darwin"])
                    .status()
                    .expect("Failed to add aarch64-apple-darwin target");

                Command::new("rustup")
                    .stdout(Stdio::inherit())
                    .stderr(Stdio::inherit())
                    .args(["target", "add", "x86_64-apple-darwin"])
                    .status()
                    .expect("Failed to add x86_64-apple-darwin target");

                Command::new("cargo")
                    .stdout(Stdio::inherit())
                    .stderr(Stdio::inherit())
                    .args([
                        "build",
                        "-p",
                        "kill_tree_cli",
                        "-r",
                        "--target",
                        "aarch64-apple-darwin",
                    ])
                    .status()
                    .expect("Failed to build kill_tree_cli");

                Command::new("cargo")
                    .stdout(Stdio::inherit())
                    .stderr(Stdio::inherit())
                    .args([
                        "build",
                        "-p",
                        "kill_tree_cli",
                        "-r",
                        "--target",
                        "x86_64-apple-darwin",
                    ])
                    .status()
                    .expect("Failed to build kill_tree_cli");
            } else {
                panic!("Unsupported platform: {platform}");
            }
        }
        Some(Commands::Test { platform }) => {
            if platform == "windows" {
                Command::new("cargo")
                    .stdout(Stdio::inherit())
                    .stderr(Stdio::inherit())
                    .args(["test", "--target", "x86_64-pc-windows-msvc"])
                    .status()
                    .expect("Failed to test kill_tree_cli");
            } else if platform == "linux" {
                Command::new("cargo")
                    .stdout(Stdio::inherit())
                    .stderr(Stdio::inherit())
                    .args(["test", "--target", "x86_64-unknown-linux-musl"])
                    .status()
                    .expect("Failed to test kill_tree_cli");
            } else if platform == "macos" {
                Command::new("cargo")
                    .stdout(Stdio::inherit())
                    .stderr(Stdio::inherit())
                    .args(["test", "--target", "aarch64-apple-darwin"])
                    .status()
                    .expect("Failed to test kill_tree_cli");

                Command::new("cargo")
                    .stdout(Stdio::inherit())
                    .stderr(Stdio::inherit())
                    .args(["test", "--target", "x86_64-apple-darwin"])
                    .status()
                    .expect("Failed to test kill_tree_cli");
            } else {
                panic!("Unsupported platform: {platform}");
            }
        }
        None => {
            panic!("No command");
        }
    }
}
