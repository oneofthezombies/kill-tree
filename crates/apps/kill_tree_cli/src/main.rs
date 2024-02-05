use clap::{
    builder::{styling::AnsiColor, Styles},
    command, value_parser, ArgAction, Parser,
};
use kill_tree::{kill_tree_with_config, Config, KillResult};

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
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let do_print = !cli.quiet;
    if do_print {
        println!(
            "Killing all of target process and its children recursively. process id: {}, signal: {}",
            cli.process_id, cli.signal
        );
    }

    let kill_results = kill_tree_with_config(
        cli.process_id,
        Config {
            signal: cli.signal.to_string(),
        },
    )?;

    if do_print {
        println!(
            "Killing is done. Number of killed processes: {}",
            kill_results.len()
        );
        for (index, kill_result) in kill_results.iter().enumerate() {
            match kill_result {
                KillResult::Killed(killed_info) => {
                    println!(
                        "[{}] Killed process. process id: {}, parent process id: {}, name: {}",
                        index,
                        killed_info.process_id,
                        killed_info.parent_process_id,
                        killed_info.name
                    );
                }
                KillResult::MaybeAlreadyTerminated(maybe_already_terminated_info) => {
                    println!(
                        "[{}] Maybe already terminated process. process id: {}, reason: {}",
                        index,
                        maybe_already_terminated_info.process_id,
                        maybe_already_terminated_info.reason
                    );
                }
                KillResult::InternalError(e) => {
                    println!("[{}] Internal error occurred. error: {}", index, e);
                }
            }
        }
    }
    Ok(())
}
