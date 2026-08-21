use super::{
    identity_from_handle_with_executable, OwnedProcessError, OwnedProcessIdentity, ProcessHandle,
};
use windows_sys::Win32::Foundation::WAIT_OBJECT_0;
use windows_sys::Win32::System::Threading::{
    TerminateProcess, WaitForSingleObject, PROCESS_QUERY_LIMITED_INFORMATION, PROCESS_TERMINATE,
};

pub(in crate::services::owned_process) fn recover_exact_with_cancel(
    expected: OwnedProcessIdentity,
    deadline: std::time::Instant,
    cancelled: &dyn Fn() -> bool,
) -> Result<(), OwnedProcessError> {
    let process = ProcessHandle::open(
        expected.pid,
        PROCESS_QUERY_LIMITED_INFORMATION | PROCESS_TERMINATE,
    )?;
    if identity_from_handle_with_executable(process.0, expected.executable)? != expected {
        return Err(OwnedProcessError::Admission);
    }
    unsafe { TerminateProcess(process.0, 1) };
    while std::time::Instant::now() < deadline {
        if cancelled() {
            return Err(OwnedProcessError::Admission);
        }
        if unsafe { WaitForSingleObject(process.0, 25) } == WAIT_OBJECT_0 {
            return Ok(());
        }
    }
    Err(OwnedProcessError::Admission)
}
