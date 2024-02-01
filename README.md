# KILL TREE

![logo](docs/images/logo.jpg)

Multi-platform library that kill the process also with all child processes. Written in Rust.  
This project is inspired by [node-tree-kill](https://github.com/pkrumins/node-tree-kill).  Thank you. 🤟  

🚧 Development of this library is currently in progress.  

## Why Did I Make

TODO

## How to Use

TODO

### Using as Rust Package

TODO

Add dependency to Cargo.toml

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

### Using as CLI Tool

#### Usages

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
