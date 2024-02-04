use clap::{
    builder::{styling::AnsiColor, Styles},
    command, value_parser, Arg, ArgAction,
};
use kill_tree::{kill_tree_with_config, Config, KillResult};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = command!()
        .name("kill-tree")
        .bin_name("kill-tree")
        .arg_required_else_help(true)
        .styles(
            Styles::styled()
                .header(AnsiColor::BrightGreen.on_default().bold())
                .usage(AnsiColor::BrightGreen.on_default().bold())
                .literal(AnsiColor::BrightCyan.on_default().bold())
                .placeholder(AnsiColor::Cyan.on_default()),
        )
        .arg(
            Arg::new("PROCESS_ID")
                .help("Process ID to kill with all children.")
                .value_parser(value_parser!(u32)),
        )
        .arg(
            Arg::new("SIGNAL")
                .help("Signal to send to the processes.")
                .default_value("SIGTERM"),
        )
        .arg(
            Arg::new("QUIET")
                .short('q')
                .long("quiet")
                .help("No logs are output.")
                .action(ArgAction::SetTrue),
        )
        .get_matches();

    let process_id = *matches.get_one::<u32>("PROCESS_ID").unwrap();
    let signal = matches.get_one::<String>("SIGNAL").unwrap();
    let quiet = *matches.get_one::<bool>("QUIET").unwrap();

    let do_print = !quiet;
    if do_print {
        println!(
            "Killing process with all children. process id: {}, signal: {}",
            process_id, signal
        );
    }

    let kill_results = kill_tree_with_config(
        process_id,
        Config {
            signal: signal.to_string(),
            ..Default::default()
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
