use super::OwnedProcessError;

pub(super) fn admit(pid: u32) -> Result<(), OwnedProcessError> {
    if pid < 2 || pid > i32::MAX as u32 {
        return Err(OwnedProcessError::Admission);
    }
    #[cfg(target_os = "macos")]
    return super::macos::admit(pid);
    #[cfg(not(target_os = "macos"))]
    {
        let group = unsafe { libc::getpgid(pid as i32) };
        (group == pid as i32)
            .then_some(())
            .ok_or(OwnedProcessError::Admission)
    }
}

pub(super) fn release(pid: u32) {
    #[cfg(target_os = "macos")]
    super::macos::release(pid);
    #[cfg(not(target_os = "macos"))]
    let _ = pid;
}

#[cfg(test)]
pub(super) fn is_confined(pid: u32) -> bool {
    #[cfg(target_os = "macos")]
    return super::macos::is_confined(pid);
    #[cfg(not(target_os = "macos"))]
    {
        pid >= 2 && unsafe { libc::getpgid(pid as i32) } == pid as i32
    }
}
