use std::env;

fn is_debug() -> bool {
    env::var("KILL_TREE_DEBUG").is_ok()
}

#[cfg(target_os = "linux")]
fn get_max_pid() -> u32 {
    0x400000
}

#[cfg(target_os = "macos")]
fn get_max_pid() -> u32 {
    99998
}

#[cfg(target_os = "windows")]
fn get_max_pid() -> u32 {
    0xFFFFFFFF
}

fn validate_pid(pid: u32) -> Result<(), Box<dyn std::error::Error>> {
    let max_pid = get_max_pid();
    if pid <= max_pid {
        Ok(())
    } else {
        Err(format!("pid is greater than max pid. pid: {}, max pid: {}", pid, max_pid).into())
    }
}

#[cfg(target_family = "unix")]
pub fn kill_tree(pid: u32) -> Result<(), Box<dyn std::error::Error>> {
    unix_impl::kill_tree(pid)
}

#[cfg(target_family = "unix")]
mod unix_impl {
    use super::*;
    use nix::{
        sys::signal::{kill, Signal},
        unistd::Pid,
    };
    use std::collections::VecDeque;

    fn parse_pid(pid: u32) -> Result<Pid, Box<dyn std::error::Error>> {
        if pid <= 0x400000 {
            Ok(Pid::from_raw(pid as i32))
        } else {
            Err(format!("Unacceptable pid value: {}", pid).into())
        }
    }

    pub(crate) fn kill_tree(pid: u32) -> Result<(), Box<dyn std::error::Error>> {
        let parsed_pid = parse_pid(pid)?;
        Ok(())
    }
}

#[cfg(target_family = "windows")]
pub fn kill_tree(pid: u32) -> Result<(), Box<dyn std::error::Error>> {
    windows_impl::kill_tree(pid)
}

#[cfg(target_family = "windows")]
mod windows_impl {
    use super::*;
    use std::collections::VecDeque;
    use windows::Win32::{
        Foundation::{CloseHandle, ERROR_NO_MORE_FILES, E_ACCESSDENIED, HANDLE},
        System::{
            Diagnostics::ToolHelp::{
                CreateToolhelp32Snapshot, Process32First, Process32Next, PROCESSENTRY32,
                TH32CS_SNAPPROCESS,
            },
            Threading::{OpenProcess, TerminateProcess, PROCESS_TERMINATE},
        },
    };

    unsafe fn close_handle(handle: HANDLE) -> Result<(), windows::core::Error> {
        unsafe {
            CloseHandle(handle).or_else(|e| {
                if is_debug() {
                    eprintln!("CloseHandle failed: {}", e);
                }
                Err(e)
            })
        }
    }

    unsafe fn get_children_pids_by_handle(
        snapshot_handle: HANDLE,
        parent_pid: u32,
    ) -> Result<Vec<u32>, windows::core::Error> {
        let mut children_pids = Vec::new();
        let mut entry = std::mem::zeroed::<PROCESSENTRY32>();
        entry.dwSize = std::mem::size_of::<PROCESSENTRY32>() as u32;
        unsafe {
            let _ = Process32First(snapshot_handle, &mut entry).or_else(|e| {
                if is_debug() {
                    eprintln!("Process32First failed: {}", e);
                }
                Err(e)
            });
            loop {
                if entry.th32ParentProcessID == parent_pid {
                    children_pids.push(entry.th32ProcessID);
                }
                if let Err(e) = Process32Next(snapshot_handle, &mut entry) {
                    if e.code() == ERROR_NO_MORE_FILES.into() {
                        break;
                    } else {
                        if is_debug() {
                            eprintln!("Process32Next failed: {}", e);
                        }
                        return Err(e);
                    }
                }
            }
        }
        Ok(children_pids)
    }

    fn get_children_pids(parent_pid: u32) -> Result<Vec<u32>, windows::core::Error> {
        unsafe {
            let handle = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0).or_else(|e| {
                if is_debug() {
                    eprintln!("CreateToolhelp32Snapshot failed: {}", e);
                }
                Err(e)
            })?;
            let result = get_children_pids_by_handle(handle, parent_pid);
            close_handle(handle)?;
            result
        }
    }

    fn terminate_process(pid: u32) -> Result<(), windows::core::Error> {
        unsafe {
            let handle = OpenProcess(PROCESS_TERMINATE, false, pid).or_else(|e| {
                if is_debug() {
                    eprintln!("OpenProcess failed. pid: {}, error: {}", pid, e);
                }
                Err(e)
            })?;
            let result = TerminateProcess(handle, 1).or_else(|e| {
                if e.code() == E_ACCESSDENIED {
                    // Access is denied.
                    // This happens when the process is already terminated.
                    // This is not an error.
                    Ok(())
                } else {
                    if is_debug() {
                        eprintln!("TerminateProcess failed. pid: {}, error: {}", pid, e);
                    }
                    Err(e)
                }
            });
            close_handle(handle)?;
            result
        }
    }

    pub(crate) fn kill_tree(pid: u32) -> Result<(), Box<dyn std::error::Error>> {
        let mut queue = VecDeque::new();
        let mut stack = Vec::new();
        queue.push_back(pid);
        stack.push(pid);
        while let Some(pid) = queue.pop_front() {
            for child_pid in get_children_pids(pid)? {
                queue.push_back(child_pid);
                stack.push(child_pid);
            }
        }
        while let Some(pid) = stack.pop() {
            terminate_process(pid)
            .and_then(|_| {
                if is_debug() {
                    eprintln!("Killed process. pid: {}", pid);
                }
                Ok(())
            }).or_else(|e| {
                if is_debug() {
                    eprintln!("Failed to kill process. pid: {}, error: {}", pid, e);
                }
                Err(e.into())
            })?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(target_family = "windows")]
    fn invalid_pid_windows() {
        assert!(kill_tree(0xFFFFFFFF).is_err());
    }

    #[test]
    #[cfg(target_family = "windows")]
    fn valid_pid_windows() {
        let child = std::process::Command::new("cmd")
            .args(&[
                "/C",
                "for /L %x in (0,0,1) do @(echo Hello & timeout /t 1 /nobreak > NUL)",
            ])
            .spawn()
            .unwrap();
        std::thread::sleep(std::time::Duration::from_secs(1));
        let pid = child.id();
        kill_tree(pid).unwrap();
    }

    #[test]
    #[cfg(target_family = "windows")]
    fn already_terminated_windows() {
        let child = std::process::Command::new("cmd")
            .args(&["/C", "echo Hello terminated"])
            .spawn()
            .unwrap();
        std::thread::sleep(std::time::Duration::from_secs(2));
        let pid = child.id();
        kill_tree(pid).unwrap();
    }

    #[test]
    #[cfg(target_family = "windows")]
    fn nested_child_windows() {
        let child = std::process::Command::new("cmd")
            .args(&["/C", "cmd /C timeout 10"])
            .spawn()
            .unwrap();
        std::thread::sleep(std::time::Duration::from_secs(3));
        let pid = child.id();
        kill_tree(pid).unwrap();
    }
}
