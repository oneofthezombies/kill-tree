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
    let mut command = Command::new(program);
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
    run("cargo", &["clippy", "--", "-D", "clippy::pedantic"]);
}

fn fmt() {
    run("cargo", &["fmt", "--", "--check"]);
}

fn build(target: &str) {
    env::set_var("RUSTFLAGS", "-C target-feature=+crt-static");
    run("rustup", &["target", "add", target]);
    run(
        "cargo",
        &["build", "-p", "kill_tree_cli", "-r", "--target", target],
    );
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
        Some(Commands::Check) => check(),
        Some(Commands::Clippy) => clippy(),
        Some(Commands::Fmt) => fmt(),
        Some(Commands::Build { target }) => build(&target),
        Some(Commands::Test { target }) => test(target),
        Some(Commands::PrePush) => pre_push(),
        None => {
            panic!("No command");
        }
    }
}
