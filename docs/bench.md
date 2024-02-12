windows kill-tree
target/release/kill_tree_cli.exe <process_id>

windows taskkill
C:\WINDOWS\system32\taskkill.exe /F /T /PID <process_id>

platform: windows, arch: x86_64, exe: target/release/kill_tree_cli.exe, count: 100, total_ms: 1616, average_ms: 16
platform: windows, arch: x86_64, exe: C:\WINDOWS\system32\taskkill.exe, count: 100, total_ms: 6167, average_ms: 61

platform: windows, arch: x86_64, exe: target/release/kill_tree_cli.exe, count: 200, total_ms: 2635, average_ms: 13
platform: windows, arch: x86_64, exe: C:\WINDOWS\system32\taskkill.exe, count: 200, total_ms: 13425, average_ms: 67

platform: windows, arch: x86_64, exe: target/release/kill_tree_cli.exe, count: 300, total_ms: 4427, average_ms: 14
platform: windows, arch: x86_64, exe: C:\WINDOWS\system32\taskkill.exe, count: 300, total_ms: 22351, average_ms: 74

```sh
# wmic cpu get NumberOfCores,NumberOfLogicalProcessors
NumberOfCores  NumberOfLogicalProcessors  
8              16
```

```sh
# systeminfo | findstr /C:"Total Physical Memory"
Total Physical Memory:     16,270 MB
```

```sh
# systeminfo | findstr /B /C:"OS Name" /B /C:"OS Version"
OS Name:                   Microsoft Windows 11 Pro
OS Version:                10.0.22621 N/A Build 22621
```

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
