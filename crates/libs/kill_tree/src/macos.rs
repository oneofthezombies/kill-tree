use crate::core::{
    Config, Error, Killable, ProcessId, ProcessIds, ProcessInfo, ProcessInfos, Result,
};
use tracing::{debug, instrument};

/// In decimal, 99998.  
pub(crate) const AVAILABLE_MAX_PROCESS_ID: u32 = 99999 - 1;

pub(crate) fn validate_process_id(process_id: ProcessId) -> Result<()> {
    crate::unix::validate_process_id(process_id, AVAILABLE_MAX_PROCESS_ID)
}

pub(crate) fn child_process_id_map_filter(_process_info: &ProcessInfo) -> bool {
    false
}

#[instrument]
pub(crate) fn get_process_info(process_id: ProcessId) -> Result<ProcessInfo> {
    let proc_bsdinfo_size = match u32::try_from(std::mem::size_of::<libproc::proc_bsdinfo>()) {
        Ok(x) => x,
        Err(e) => {
            return Err(Error::InvalidCast {
                reason: "Failed to convert size of proc_bsdinfo to u32".into(),
                source: e,
            });
        }
    };
    let proc_bsdinfo_size_sign = match i32::try_from(proc_bsdinfo_size) {
        Ok(x) => x,
        Err(e) => {
            return Err(Error::InvalidCast {
                reason: "Failed to convert size of proc_bsdinfo to i32".into(),
                source: e,
            });
        }
    };
    let mut proc_bsdinfo = unsafe { std::mem::zeroed::<libproc::proc_bsdinfo>() };
    let proc_pidtbsdinfo_sign = match i32::try_from(libproc::PROC_PIDTBSDINFO) {
        Ok(x) => x,
        Err(e) => {
            return Err(Error::InvalidCast {
                reason: "Failed to convert PROC_PIDTBSDINFO to i32".into(),
                source: e,
            });
        }
    };
    let process_id_sign = match i32::try_from(process_id) {
        Ok(x) => x,
        Err(e) => {
            return Err(Error::InvalidCast {
                reason: "Failed to convert process id to i32".into(),
                source: e,
            });
        }
    };
    let result = unsafe {
        libproc::proc_pidinfo(
            process_id_sign,
            proc_pidtbsdinfo_sign,
            0,
            std::ptr::addr_of_mut!(proc_bsdinfo).cast::<std::ffi::c_void>(),
            proc_bsdinfo_size_sign,
        )
    };
    if result <= 0 {
        return Err(std::io::Error::last_os_error().into());
    }
    let name = unsafe { std::ffi::CStr::from_ptr(std::ptr::addr_of!(proc_bsdinfo.pbi_name[0])) }
        .to_string_lossy()
        .to_string();
    Ok(ProcessInfo {
        process_id,
        parent_process_id: proc_bsdinfo.pbi_ppid,
        name,
    })
}

#[instrument]
pub(crate) fn get_process_ids() -> Result<ProcessIds> {
    let buffer_size_sign =
        unsafe { libproc::proc_listpids(libproc::PROC_ALL_PIDS, 0_u32, std::ptr::null_mut(), 0) };
    if buffer_size_sign <= 0 {
        return Err(std::io::Error::last_os_error().into());
    }
    let buffer_size = match usize::try_from(buffer_size_sign) {
        Ok(x) => x,
        Err(e) => {
            return Err(Error::InvalidCast {
                reason: "Failed to convert buffer size to usize".into(),
                source: e,
            });
        }
    };
    let mut buffer = vec![0; buffer_size];
    let result = unsafe {
        libproc::proc_listpids(
            libproc::PROC_ALL_PIDS,
            0_u32,
            buffer.as_mut_ptr().cast(),
            buffer_size_sign,
        )
    };
    if result <= 0 {
        return Err(std::io::Error::last_os_error().into());
    }
    let process_ids = buffer.as_slice();
    Ok(process_ids.to_vec())
}

#[instrument]
pub(crate) fn get_process_infos() -> Result<ProcessInfos> {
    let process_ids = crate::macos::get_process_ids()?;
    let mut process_infos = ProcessInfos::new();
    for process_id in process_ids {
        let process_info = match crate::macos::get_process_info(process_id) {
            Ok(x) => x,
            Err(e) => {
                debug!(error = ?e, "Failed to get process info");
                continue;
            }
        };
        process_infos.push(process_info);
    }
    Ok(process_infos)
}

pub(crate) fn new_killer(config: &Config) -> Result<impl Killable> {
    crate::unix::new_killer(config)
}

#[cfg(feature = "blocking")]
pub(crate) mod blocking {
    use super::{ProcessInfos, Result};

    pub(crate) fn get_process_infos() -> Result<ProcessInfos> {
        crate::macos::get_process_infos()
    }
}

#[cfg(feature = "tokio")]
pub(crate) mod tokio {
    use super::{ProcessInfos, Result};

    pub(crate) async fn get_process_infos() -> Result<ProcessInfos> {
        crate::macos::get_process_infos()
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::kill_tree;

//     #[tokio::test]
//     async fn process_id_max_plus_1() {
//         let result = kill_tree(AVAILABLE_MAX_PROCESS_ID + 1).await;
//         assert!(result.is_err());
//         assert_eq!(
//             result.unwrap_err().to_string(),
//             "Process id is too large. process id: 99999, available max process id: 99998"
//         );
//     }
// }

#[allow(warnings)]
#[allow(clippy::all)]
#[allow(clippy::pedantic)]
mod libproc {
    include!(concat!(env!("OUT_DIR"), "/libproc_bindings.rs"));
}
