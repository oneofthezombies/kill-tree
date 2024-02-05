use clap::{
    builder::{styling::AnsiColor, Styles},
    command, value_parser, ArgAction, Parser,
};
use kill_tree::kill_tree_with_signal;

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

    let outputs = match kill_tree_with_signal(cli.process_id, cli.signal.as_str()).await {
        Ok(x) => x,
        Err(e) => {
            if do_print {
                println!("Failed to kill processes. error: {}", e);
            }
            let e: Box<dyn std::error::Error> = e;
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
                kill_tree::tree::Output::Killed {
                    process_id,
                    parent_process_id,
                    name,
                } => {
                    println!(
                        "[{}] Killed process. process id: {}, parent process id: {}, name: {}",
                        index, process_id, parent_process_id, name
                    );
                }
                kill_tree::tree::Output::MaybeAlreadyTerminated { process_id, reason } => {
                    println!(
                        "[{}] Maybe already terminated process. process id: {}, reason: {}",
                        index, process_id, reason
                    );
                }
            }
        }
    }
    Ok(())
}
