# KILL TREE

![logo](docs/images/logo.jpg)

Multi-platform library that kill the process also with all child processes. Written in Rust.  
This project is inspired by [node-tree-kill](https://github.com/pkrumins/node-tree-kill).  Thank you. 🤟  

🚧 Development of this library is currently in progress.  

## Why Did I Make

## How To Use

### As a Rust Library

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

### As a CLI Program

### As a Node.js Package

```sh
# using npm
npm install kill-tree
```
