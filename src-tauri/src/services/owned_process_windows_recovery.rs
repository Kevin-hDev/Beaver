use super::{identity_from_handle, is_in_owned_job, ProcessHandle};
use crate::services::owned_process::{OwnedProcessError, OwnedProcessInspection};
use windows_sys::Win32::System::Threading::PROCESS_QUERY_LIMITED_INFORMATION;

pub(super) fn inspect_for_recovery(pid: u32) -> Result<OwnedProcessInspection, OwnedProcessError> {
    if pid < 2 {
        return Err(OwnedProcessError::Admission);
    }
    let process = ProcessHandle::open(pid, PROCESS_QUERY_LIMITED_INFORMATION)?;
    if !is_in_owned_job(process.0)? {
        // Hors du Job Beaver est une identité différente certaine, pas une lecture ambiguë.
        return Ok(OwnedProcessInspection::Unowned);
    }
    identity_from_handle(process.0).map(OwnedProcessInspection::Owned)
}
