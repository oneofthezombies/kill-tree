use std::io;

use clap::{
    builder::{styling::AnsiColor, Styles},
    command, value_parser, ArgAction, Parser,
};
use kill_tree::{blocking::kill_tree_with_config, Config};
use tracing::{
    subscriber::{self, SetGlobalDefaultError},
    Level,
};
use tracing_subscriber::FmtSubscriber;

fn get_styles() -> Styles {
    Styles::styled()
        .header(AnsiColor::BrightGreen.on_default().bold())
        .usage(AnsiColor::BrightGreen.on_default().bold())
        .literal(AnsiColor::BrightCyan.on_default().bold())
        .placeholder(AnsiColor::Cyan.on_default())
}

#[derive(Parser)]
#[command(name = "kill-tree")]
#[command(bin_name = "kill-tree")]
#[command(arg_required_else_help = true)]
#[command(styles = get_styles())]
#[command(author, version, about, long_about=None)]
struct Cli {
    #[arg(help = "Process ID to kill with all children.")]
    #[arg(value_parser = value_parser!(u32))]
    process_id: u32,

    #[arg(help = "Signal to send to the processes.")]
    #[arg(default_value = "SIGTERM")]
    signal: String,

    #[arg(short, long)]
    #[arg(help = "No logs are output.")]
    #[arg(action = ArgAction::SetTrue)]
    quiet: bool,

    #[arg(long)]
    #[arg(help = "Set the log level. Available levels: error, warn, info, debug, trace")]
    #[arg(default_value = "warn")]
    #[arg(value_parser = value_parser!(Level))]
    log_level: Level,
}

fn init_log(level: Level) -> Result<(), SetGlobalDefaultError> {
    subscriber::set_global_default(FmtSubscriber::builder().with_max_level(level).finish())
}

fn main() -> kill_tree::Result<()> {
    let cli = Cli::parse();
    let do_print = !cli.quiet;
    if do_print {
        init_log(cli.log_level).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        println!(
            "Killing all of target process and its children recursively. process id: {}, signal: {}",
            cli.process_id, cli.signal
        );
    }

    let outputs = match kill_tree_with_config(
        cli.process_id,
        &Config {
            signal: cli.signal,
            ..Default::default()
        },
    ) {
        Ok(x) => x,
        Err(e) => {
            if do_print {
                println!("Failed to kill processes. error: {e}");
            }
            return Err(e);
        }
    };

    if do_print {
        println!(
            "Killing is done. Number of killed processes: {}",
            outputs.len()
        );
        for (index, output) in outputs.iter().enumerate() {
            match output {
                kill_tree::Output::Killed {
                    process_id,
                    parent_process_id,
                    name,
                } => {
                    println!(
                        "[{index}] Killed process. process id: {process_id}, parent process id: {parent_process_id}, name: {name}"
                    );
                }
                kill_tree::Output::MaybeAlreadyTerminated { process_id, source } => {
                    println!(
                        "[{index}] Maybe already terminated process. process id: {process_id}, source: {source}"
                    );
                }
            }
        }
    }
    Ok(())
}
