# Kill Tree

![logo](docs/images/logo.jpg)

Multi-platform library that kill the process also with all child processes. Written in Rust.  
This project is inspired by [node-tree-kill](https://github.com/pkrumins/node-tree-kill).  Thank you. 🤟  

🚧 Development of this library is currently in progress.  

## How to install

```shell
cargo add kill-tree
```

## How to use

```rust
kill_tree(pid)?
```

```rust
kill_tree_with_config(config)?
```
