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

⚠️ _Signals are ignored on Windows platforms._

```sh
kill-tree 777
```

If you want to send another signal, you can enter that signal as the second parameter.  
Below is an example of sending a SIGKILL signal to a process with process ID 777 and to all child processes.  

⚠️ _Also, signals are ignored on Windows platforms._

```sh
kill-tree 777 SIGKILL
```

### Using as Rust Library

🔖 TODO

Add `kill-tree` dependency to Cargo.toml

```toml
# Cargo.toml
[dependencies]
kill-tree = "0.1"
```

```rust
kill_tree(pid)?
```

```rust
kill_tree_with_config(config)?
```
