# KILL TREE

![logo](docs/images/logo.jpg)

A library and CLI tool that kills all of target process and its children recursively not depending on the other commands such as `kill`, `ps`, `pgrep`, `taskkill` or `wmic`.  
This is written in Rust and powered by [Tokio](https://github.com/tokio-rs/tokio).  
This project is inspired by [node-tree-kill](https://github.com/pkrumins/node-tree-kill).  Thank you. 🤟  

🚧 Development of this library is currently in progress.  

## Why Did I Make

🔖 TODO

## How to Use

### Using as CLI Tool

Below is an example of sending SIGTERM signals to a process with process ID 777, and to all child processes.  

ℹ️ _Signals are ignored on Windows platforms._

```sh
kill-tree 777
```

If you want to send another signal, you can enter that signal as the second parameter.  
Below is an example of sending a SIGKILL signal to a process with process ID 777 and to all child processes.  

ℹ️ _Also, signals are ignored on Windows platforms._

```sh
kill-tree 777 SIGKILL
```

### Using as Rust Library

> ⚠️ This library must be called in Tokio runtime.  

Add `kill-tree` to your dependencies.

```toml
# Cargo.toml
[dependencies]
kill-tree = "0.1"
```

kill process and its children recursively with default signal `SIGTERM`.

```rust
use kill_tree::kill_tree;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let process_id = 12345;
    let outputs = kill_tree(process_id).await.map_err(|e| e.to_string())?;
    for output in outputs {
        match output {
            kill_tree::tree::Output::Killed {
                process_id,
                parent_process_id,
                name,
            } => {
                println!(
                    "Killed process. process id: {}, parent process id: {}, name: {}",
                    process_id, parent_process_id, name
                );
            }
            kill_tree::tree::Output::MaybeAlreadyTerminated { process_id, reason } => {
                println!(
                    "Maybe already terminated process. process id: {}, reason: {}",
                    process_id, reason
                );
            }
        }
    }
    Ok(())
}
```

kill process and its children recursively with signal `SIGKILL`.

```rust
use kill_tree::kill_tree_with_signal;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let process_id = 12345;
    let outputs = kill_tree_with_signal(process_id, "SIGKILL")
        .await
        .map_err(|e| e.to_string())?;
    for output in outputs {
        match output {
            kill_tree::tree::Output::Killed {
                process_id,
                parent_process_id,
                name,
            } => {
                println!(
                    "Killed process. process id: {}, parent process id: {}, name: {}",
                    process_id, parent_process_id, name
                );
            }
            kill_tree::tree::Output::MaybeAlreadyTerminated { process_id, reason } => {
                println!(
                    "Maybe already terminated process. process id: {}, reason: {}",
                    process_id, reason
                );
            }
        }
    }
    Ok(())
}
```
