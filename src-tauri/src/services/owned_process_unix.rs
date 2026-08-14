use super::OwnedProcessError;
use super::OwnedProcessIdentity;
#[cfg(target_os = "linux")]
use std::fs;

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

pub(super) fn identity(pid: u32) -> Result<OwnedProcessIdentity, OwnedProcessError> {
    if pid < 2 || pid > i32::MAX as u32 {
        return Err(OwnedProcessError::Admission);
    }
    let scope = unsafe { libc::getpgid(pid as i32) };
    if scope <= 0 {
        return Err(OwnedProcessError::Admission);
    }
    let native_start_time = start_time(pid).ok_or(OwnedProcessError::Admission)?;
    let executable = executable_identity(pid).ok_or(OwnedProcessError::Admission)?;
    Ok(OwnedProcessIdentity {
        pid,
        native_scope: scope as u64,
        native_start_time,
        executable,
    })
}

pub(super) fn identity_with_executable(
    pid: u32,
    executable: u128,
) -> Result<OwnedProcessIdentity, OwnedProcessError> {
    if pid < 2 || pid > i32::MAX as u32 || executable == 0 {
        return Err(OwnedProcessError::Admission);
    }
    let scope = unsafe { libc::getpgid(pid as i32) };
    if scope <= 0 {
        return Err(OwnedProcessError::Admission);
    }
    Ok(OwnedProcessIdentity {
        pid,
        native_scope: scope as u64,
        native_start_time: start_time(pid).ok_or(OwnedProcessError::Admission)?,
        executable,
    })
}

pub(super) fn identity_matches(expected: OwnedProcessIdentity) -> Result<(), OwnedProcessError> {
    (identity(expected.pid)? == expected)
        .then_some(())
        .ok_or(OwnedProcessError::Admission)
}

pub(super) fn release(pid: u32) {
    #[cfg(target_os = "macos")]
    super::macos::release(pid);
    #[cfg(not(target_os = "macos"))]
    let _ = pid;
}

#[cfg(target_os = "linux")]
fn start_time(pid: u32) -> Option<u64> {
    let bytes = fs::read(format!("/proc/{pid}/stat")).ok()?;
    let end = bytes.iter().rposition(|byte| *byte == b')')?;
    bytes
        .get(end + 2..)?
        .split(|byte| *byte == b' ')
        .filter(|field| !field.is_empty())
        .nth(19)?
        .iter()
        .try_fold(0_u64, |value, byte| {
            value.checked_mul(10)?.checked_add(u64::from(byte - b'0'))
        })
}

#[cfg(target_os = "macos")]
fn start_time(pid: u32) -> Option<u64> {
    super::macos::read_start_time(pid)
}

#[cfg(target_os = "linux")]
fn executable_identity(pid: u32) -> Option<u128> {
    use std::os::unix::fs::MetadataExt;
    let metadata = fs::metadata(format!("/proc/{pid}/exe")).ok()?;
    Some((u128::from(metadata.dev()) << 64) | u128::from(metadata.ino()))
}

#[cfg(target_os = "macos")]
fn executable_identity(pid: u32) -> Option<u128> {
    super::macos::read_executable_identity(pid)
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
