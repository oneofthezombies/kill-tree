use nix::errno::Errno;

use crate::common::{ProcessInfo, TreeKiller};
use std::{error::Error, ffi::c_void, io, ptr};

const AVAILABLE_MAX_PROCESS_ID: u32 = 99999 - 1;

impl TreeKiller {
    pub(crate) fn validate_pid(&self) -> Result<(), Box<dyn Error>> {
        self.validate_pid_with_available_max(AVAILABLE_MAX_PROCESS_ID)
    }

    pub(crate) fn get_process_infos(&self) -> Result<Vec<ProcessInfo>, Box<dyn Error>> {
        let buffer_size =
            unsafe { libproc::proc_listpids(libproc::PROC_ALL_PIDS, 0 as u32, ptr::null_mut(), 0) };
        if buffer_size <= 0 {
            return Err(io::Error::last_os_error().into());
        }
        let mut buffer = vec![0; buffer_size as usize];
        let result = unsafe {
            libproc::proc_listpids(
                libproc::PROC_ALL_PIDS,
                0 as u32,
                buffer.as_mut_ptr() as *mut c_void,
                buffer_size,
            )
        };
        if result <= 0 {
            return Err(io::Error::last_os_error().into());
        }
        let process_ids = buffer.as_slice();
        let mut process_infos = Vec::new();
        for &process_id in process_ids {
            let mut proc_bsdinfo = unsafe { std::mem::zeroed::<libproc::proc_bsdinfo>() };
            let proc_bsdinfo_size = std::mem::size_of::<libproc::proc_bsdinfo>() as u32;
            let result = unsafe {
                libproc::proc_pidinfo(
                    process_id,
                    libproc::PROC_PIDTBSDINFO as i32,
                    0,
                    &mut proc_bsdinfo as *mut _ as *mut c_void,
                    proc_bsdinfo_size as i32,
                )
            };
            if result <= 0 {
                let error = io::Error::last_os_error();
                if let Some(os_error_code) = error.raw_os_error() {
                    if os_error_code == Errno::ESRCH as i32 {
                        // ESRCH: No such process.
                        // This happens when the process has already terminated.
                        // This is not an error.
                        continue;
                    } else if os_error_code == Errno::EPERM as i32 {
                        // EPERM: Operation not permitted.
                        // This happens when the process is owned by another user.
                        // This is not an error.
                        continue;
                    }
                }
                return Err(error.into());
            }
            process_infos.push(ProcessInfo {
                process_id: process_id as u32,
                parent_process_id: proc_bsdinfo.pbi_ppid,
            });
        }
        Ok(process_infos)
    }
}

#[allow(warnings)]
mod libproc {
    include!(concat!(env!("OUT_DIR"), "/libproc_bindings.rs"));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{common::Config, kill_tree_with_config};

    #[test]
    fn process_id_max_plus_1() {
        let result = kill_tree_with_config(AVAILABLE_MAX_PROCESS_ID + 1, Config::default());
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Process id is too large. process id: 99999, available max process id: 99998"
        );
    }
}
