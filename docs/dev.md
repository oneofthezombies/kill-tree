# Development environment

## Windows

host information

```sh
# systeminfo | findstr /B /C:"OS Name" /B /C:"OS Version"
OS Name:                   Microsoft Windows 11 Pro
OS Version:                10.0.22621 N/A Build 22621
```

rust compiler information

```sh
# rustc -vV
rustc 1.75.0 (82e1608df 2023-12-21)
binary: rustc
commit-hash: 82e1608dfa6e0b5569232559e3d385fea5a93112
commit-date: 2023-12-21
host: x86_64-pc-windows-msvc
release: 1.75.0
LLVM version: 17.0.6
```

## Linux

host information

```sh
# hostnamectl | grep -E 'Chassis:|Virtualization:|Operating System:|Kernel:|Architecture:'
         Chassis: container
  Virtualization: wsl
Operating System: Ubuntu 22.04.3 LTS
          Kernel: Linux 5.15.133.1-microsoft-standard-WSL2
    Architecture: x86-64
```

rust compiler information

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

## Macos

TODO

```sh
# rustc -vV
```
