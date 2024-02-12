# Kill Tree vs Taskkill

## Results

| App | Kill Count | Total (ms) | Average (ms) | Faster Than Taskkill |
| --- | --- | --- | --- | --- | 
| Kill Tree | 100 | 1616 | 16 | 3.8x |
| taskkill | 100 | 6167 | 61 | 1x |

| App | Kill Count | Total (ms) | Average (ms) | Faster Than Taskkill |
| --- | --- | --- | --- | --- | 
| Kill Tree | 200 | 2635 | 13 | 5x |
| taskkill | 200 | 13425 | 67 | 1x |

| App | Kill Count | Total (ms) | Average (ms) | Faster Than Taskkill |
| --- | --- | --- | --- | --- | 
| Kill Tree | 300 | 4427 | 14 | 5x |
| taskkill | 300 | 22351 | 74 | 1x |

## Executable

### Kill Tree

build: `cargo build --package kill_tree_cli --bins --release`  
executable: `target/release/kill_tree_cli.exe` <process_id>

### Taskkill

excutable: `C:\WINDOWS\system32\taskkill.exe /F /T /PID` <process_id>

## Environment

### CPU

```sh
# wmic cpu get NumberOfCores,NumberOfLogicalProcessors
NumberOfCores  NumberOfLogicalProcessors  
8              16
```

### Memory

```sh
# systeminfo | findstr /C:"Total Physical Memory"
Total Physical Memory:     16,270 MB
```

### OS Version

```sh
# systeminfo | findstr /B /C:"OS Name" /B /C:"OS Version"
OS Name:                   Microsoft Windows 11 Pro
OS Version:                10.0.22621 N/A Build 22621
```

### Rust Compiler Version

```sh
# rustc -vV
rustc 1.78.0-nightly (1a648b397 2024-02-11)
binary: rustc
commit-hash: 1a648b397dedc98ada3dd3360f6d661ec2436c56
commit-date: 2024-02-11
host: x86_64-pc-windows-msvc
release: 1.78.0-nightly
LLVM version: 17.0.6
```
