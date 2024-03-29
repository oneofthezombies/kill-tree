# Development environment

## Host Platform

### Windows

#### Host Information

```sh
# systeminfo | findstr /B /C:"OS Name" /B /C:"OS Version"
OS Name:                   Microsoft Windows 11 Pro
OS Version:                10.0.22621 N/A Build 22621
```

#### Rust Compiler Information

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

### Linux

#### Host Information

```sh
# hostnamectl | grep -E 'Chassis:|Virtualization:|Operating System:|Kernel:|Architecture:'
         Chassis: container
  Virtualization: wsl
Operating System: Ubuntu 22.04.3 LTS
          Kernel: Linux 5.15.133.1-microsoft-standard-WSL2
    Architecture: x86-64
```

#### Rust Compiler Information

```sh
# rustc -vV
rustc 1.75.0 (82e1608df 2023-12-21)
binary: rustc
commit-hash: 82e1608dfa6e0b5569232559e3d385fea5a93112
commit-date: 2023-12-21
host: x86_64-unknown-linux-gnu
release: 1.75.0
LLVM version: 17.0.6
```

#### Requirements

```sh
sudo apt install lld
rustup target add x86_64-unknown-linux-musl
```

### Macos

#### Host Information

```sh
# sw_vers
ProductName:		macOS
ProductVersion:		14.2.1
BuildVersion:		23C71
```

```sh
# uname -v
Darwin Kernel Version 23.2.0: Wed Nov 15 21:53:34 PST 2023; root:xnu-10002.61.3~2/RELEASE_ARM64_T8103
```

#### Rust Compiler Information

```sh
# rustc -vV
rustc 1.75.0 (82e1608df 2023-12-21)
binary: rustc
commit-hash: 82e1608dfa6e0b5569232559e3d385fea5a93112
commit-date: 2023-12-21
host: aarch64-apple-darwin
release: 1.75.0
LLVM version: 17.0.6
```

# Test environment

## Requirements

### Node.js

Use to create multiple child processes in a multi-platform environment.  
`sh`, `Batch` or `Powershell` scripts are not used.  
Call `node` command from the unit tests.  
Please ensure `node` command.

#### Version

```sh
# node --version
v20.11.0
```
