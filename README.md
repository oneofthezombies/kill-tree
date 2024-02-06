# Kill Tree

![logo](docs/images/logo.jpg)

🌳 Kill Tree is a library and CLI tool that kills all of target process and its children recursively not depending on the other commands such as kill, taskkill.  
This is written in Rust and powered by [Tokio](https://github.com/tokio-rs/tokio).  
This project is inspired by [node-tree-kill](https://github.com/pkrumins/node-tree-kill).  Thank you. 🤟  

[![Build Status][actions-badge]][actions-url]
[![Crates.io][crates-badge]][crates-url]

[actions-badge]: https://github.com/oneofthezombies/kill-tree/workflows/CI/badge.svg
[actions-url]: https://github.com/oneofthezombies/kill-tree/actions?query=workflow?CI+branch=main
[crates-badge]: https://img.shields.io/crates/v/kill-tree.svg
[crates-url]: https://crates.io/crates/kill-tree

## Why Did I Make

This is my first Rust crate and CLI tool. it means that I started because it was a suitable project to development with Rust.  
There was no multi-platform library and CLI to allow the process to be terminated recursively.  

In the past, various types of CLI tools had to be invoked to implement features in the Node.js project.  
It was a project that should be able to run shell scripts for general purposely like CI/CD tools.  
I had an issue in Node.js environment where I create a child process on the Windows platform, I run an application or a batch script through `cmd` and the grandchild process would not be terminated.  
The reason is that the Windows platform does not have _Signal_.  
I had to call `taskkill` to cancel or force kill nested shell scripts that user ran on Windows platforms.  
And when I terminate the application I distributed, I wanted all of the child processes to be terminate.  
So, I solved it using a library called `tree-kill` in Npm.js. functionally, I was satisfied.  

However, if I need this feature for similar reasons next time, I will need to install Node.js and I fetch `tree-kill` library and run it.  
This dependence is quite deep. because internally, this invoke CLI tools such as `taskkill`.
In summary, Node.js runtime (including npm package manager), `tree-kill` package and `taskkill` CLI tool are required.  
If I run this with an `npx` or wrapped Node.js script, it takes quite a while.  
because it load and evaluate the script and call `taskkill` from the script to the child process.  

And it is often necessary to terminate the process in a terminal environment, with different commands for each platform.  
For example, `kill -9 777` or `taskkill /T /F /PID 777`.  

So I made it possible to recursively kill the processes directly from the code if it was a Rust project.  
And the interface was unified because the CLI tool that kill the process varies from platform to platform.  

## Why use Rust

Rust is a _battery included_ system programming language that does not have a garbage collector and includes package manager, basic package registry and a variety of other utility features.  
And it supports asynchronous grammer at the language level so that you don't need a green thread.  

I started programming in C language, and I used C++ as my main programming language.  
And I also used Go, JavaScript (including Typescript, Node.js).  
The C and C++ language do not have a package manager and a basic package registry. (There is, but I didn't see it as mainstream.)  
Node.js, the JavaScript runtime, operates on an event basis. but an event loop works in one thread and you have to have multiple event loops to use multiple threads.  
The Go programming language is _battery included_ and has a green thread _goroutine_ and a channel for communication between goroutine.  
If I didn't know Rust, I might have compromised in Go language. 🙂  

This is why I use Rust, and it hasn't been long since I used it, so I think I'll try more.

## Why use Tokio

I like _Task based parallelism_.  

Tokio is asynchronous runtime framework written in Rust. and it's one of the most widely used.  
In the current environment where _Standard async runtime_ is not yet available, I think using Tokio is a pretty reasonable option.  
Tokio's robust and many features help me focus on my business logic.  

When I first learned programming, I learned _Role based parallelism_, and I use this.  
For example, UI thread (main thread), http request thread, loading thread, etc.  
This approach is not uniform throughput between threads and can be heavy on certain threads.  
However, _Task based parallelism_ is likely to equalize throughput between threads so that computing resources are not wasted.  

Tokio runtime written in Rust allows I to work on __Zero cost task based parallelism__ without having to pay attention to detail implementation.  

## How to Use

### Using as CLI Tool

Below is an example of sending `SIGTERM` signals to a process with process ID `777`, and to all child processes.  

ℹ️ _Signals are ignored on Windows platforms._

```sh
kill-tree 777
```

If you want to send another signal, you can enter that signal as the second parameter.  
Below is an example of sending a `SIGKILL` signal to a process with process ID `777` and to all child processes.  

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

Kill process and its children recursively with default signal `SIGTERM`.  
Returns a list of process information when a function is called.  
Process information is `Killed` or `MaybeAlreadyTerminated`.  
If process information is `Killed` type, it has `process_id`, `parent_process_id` and `name`.  
Or `MaybeAlreadyTerminated` type, it has `process_id`, `reason`.  

There are two types because they can be killed during the process of querying and killing processes.  
So, when this query or kill a process, consider it a success even if it fails.  
This is because the purpose of this library is to make the process `not exist` state.

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
                    "Killed process. process id: {process_id}, parent process id: {parent_process_id}, name: {name}"
                );
            }
            kill_tree::tree::Output::MaybeAlreadyTerminated { process_id, reason } => {
                println!(
                    "Maybe already terminated process. process id: {process_id}, reason: {reason}"
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
                    "Killed process. process id: {process_id}, parent process id: {parent_process_id}, name: {name}"
                );
            }
            kill_tree::tree::Output::MaybeAlreadyTerminated { process_id, reason } => {
                println!(
                    "Maybe already terminated process. process id: {process_id}, reason: {reason}"
                );
            }
        }
    }
    Ok(())
}
```

## Support Platform and Architecture

| Platform | Architecture | Support |
| --- | --- | --- |
| Windows | x86_64 | ✅ |
| Windows | aarch64 | Not tested |
| Linux | x86_64 | ✅ |
| Linux | aarch64 | Not tested |
| Macos | x86_64 | ✅ |
| Macos | aarch64 | ✅ |

This CLI and library depend on an operating system's system library.  
Because it's the operating system that owns the processes.

| Platform | Dependencies |
| --- | --- |
| Windows | kernel32.dll |
| | oleaut32.dll |
| | ntdll.dll |
| | advapi32.dll | 
| | bcrypt.dll |
| Linux | - |
| Macos | libiconv.dylib |
| | libSystem.dylib |
