use crate::{
    common::{self, Impl, ProcessInfo, ProcessInfos},
    ProcessId,
};
use std::{
    ffi::{c_void, CStr},
    io, ptr,
};
use tokio::task::JoinSet;
use tracing::{debug, instrument};

const AVAILABLE_MAX_PROCESS_ID: u32 = 99999 - 1;

#[instrument]
pub(crate) async fn get_process_info(process_id: ProcessId) -> Option<ProcessInfo> {
    let proc_bsdinfo_size = match u32::try_from(std::mem::size_of::<libproc::proc_bsdinfo>()) {
        Ok(x) => x,
        Err(e) => {
            debug!(error = ?e, "failed to convert size of proc_bsdinfo");
            return None;
        }
    };
    let mut proc_bsdinfo = unsafe { std::mem::zeroed::<libproc::proc_bsdinfo>() };
    let result = unsafe {
        libproc::proc_pidinfo(
            process_id as i32,
            libproc::PROC_PIDTBSDINFO as i32,
            0,
            &mut proc_bsdinfo as *mut _ as *mut c_void,
            proc_bsdinfo_size as i32,
        )
    };
    if result <= 0 {
        let error = io::Error::last_os_error();
        debug!(error = ?error, process_id, "failed to get process info");
        return None;
    }
    let name = unsafe { CStr::from_ptr(&proc_bsdinfo.pbi_name[0] as *const i8) }
        .to_string_lossy()
        .to_string();
    Some(ProcessInfo {
        process_id,
        parent_process_id: proc_bsdinfo.pbi_ppid,
        name,
    })
}

#[instrument]
pub(crate) async fn get_process_infos() -> common::Result<ProcessInfos> {
    let buffer_size =
        unsafe { libproc::proc_listpids(libproc::PROC_ALL_PIDS, 0_u32, ptr::null_mut(), 0) };
    if buffer_size <= 0 {
        return Err(io::Error::last_os_error().into());
    }
    let mut buffer = vec![0; buffer_size as usize];
    let result = unsafe {
        libproc::proc_listpids(
            libproc::PROC_ALL_PIDS,
            0_u32,
            buffer.as_mut_ptr().cast(),
            buffer_size,
        )
    };
    if result <= 0 {
        return Err(io::Error::last_os_error().into());
    }
    let process_ids = buffer.as_slice();
    let mut tasks: JoinSet<Option<ProcessInfo>> = JoinSet::new();
    for &process_id_sign in process_ids {
        let process_id = match u32::try_from(process_id_sign) {
            Ok(x) => x,
            Err(e) => {
                debug!(error = ?e, "failed to convert process id");
                continue;
            }
        };
        tasks.spawn(get_process_info(process_id));
    }
    let mut process_infos = ProcessInfos::new();
    while let Some(result) = tasks.join_next().await {
        let process_info = match result {
            Ok(x) => x,
            Err(e) => {
                debug!(error = ?e, "failed to get process info");
                continue;
            }
        };
        if let Some(process_info) = process_info {
            process_infos.push(process_info);
        }
    }
    Ok(process_infos)
}

impl Impl {
    pub(crate) fn validate_process_id(&self) -> common::Result<()> {
        crate::unix::validate_process_id(self.process_id, AVAILABLE_MAX_PROCESS_ID)
    }

    pub(crate) async fn get_process_infos(&self) -> common::Result<ProcessInfos> {
        get_process_infos().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kill_tree;

    #[tokio::test]
    async fn process_id_max_plus_1() {
        let result = kill_tree(AVAILABLE_MAX_PROCESS_ID + 1).await;
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Process id is too large. process id: 99999, available max process id: 99998"
        );
    }
}

#[allow(warnings)]
#[allow(clippy::pedantic)]
mod libproc {
    include!(concat!(env!("OUT_DIR"), "/libproc_bindings.rs"));
}
