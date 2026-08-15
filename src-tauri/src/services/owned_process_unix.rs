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

pub(super) fn recover_exact(
    expected: OwnedProcessIdentity,
    deadline: std::time::Instant,
) -> Result<(), OwnedProcessError> {
    let current = identity(expected.pid)?;
    if current != expected {
        return Err(OwnedProcessError::Admission);
    }
    unsafe {
        libc::kill(-(expected.native_scope as libc::pid_t), libc::SIGTERM);
        libc::kill(expected.pid as libc::pid_t, libc::SIGTERM);
    }
    while std::time::Instant::now() < deadline {
        match wait_for_recovery(expected.pid)? {
            Some(()) => {
                release(expected.pid);
                return Ok(());
            }
            None => std::thread::yield_now(),
        }
    }
    unsafe {
        libc::kill(-(expected.native_scope as libc::pid_t), libc::SIGKILL);
        libc::kill(expected.pid as libc::pid_t, libc::SIGKILL);
    }
    wait_for_recovery_blocking(expected.pid)?;
    release(expected.pid);
    Ok(())
}

pub(super) fn signal_exact(
    expected: OwnedProcessIdentity,
    force: bool,
) -> Result<(), OwnedProcessError> {
    #[cfg(target_os = "linux")]
    {
        let fd = pidfd_open(expected.pid)?;
        let result = (identity(expected.pid)? == expected)
            .then_some(())
            .ok_or(OwnedProcessError::Admission)
            .and_then(|()| pidfd_signal(fd, if force { libc::SIGKILL } else { libc::SIGTERM }));
        unsafe { libc::close(fd) };
        return result;
    }
    #[cfg(target_os = "macos")]
    {
        let current = identity(expected.pid)?;
        if current != expected || current.native_scope != expected.pid as u64 {
            return Err(OwnedProcessError::Admission);
        }
        let signal = if force { libc::SIGKILL } else { libc::SIGTERM };
        let result = unsafe { libc::kill(-(expected.native_scope as libc::pid_t), signal) };
        (result == 0)
            .then_some(())
            .ok_or(OwnedProcessError::Admission)
    }
}

pub(super) fn process_exists(pid: u32) -> bool {
    if pid < 2 || pid > i32::MAX as u32 {
        return false;
    }
    let result = unsafe { libc::kill(pid as libc::pid_t, 0) };
    result == 0 || std::io::Error::last_os_error().raw_os_error() != Some(libc::ESRCH)
}

#[cfg(target_os = "linux")]
fn pidfd_open(pid: u32) -> Result<libc::c_int, OwnedProcessError> {
    let fd = unsafe { libc::syscall(libc::SYS_pidfd_open, pid, 0) } as libc::c_int;
    (fd >= 0).then_some(fd).ok_or(OwnedProcessError::Admission)
}

#[cfg(target_os = "linux")]
fn pidfd_signal(fd: libc::c_int, signal: libc::c_int) -> Result<(), OwnedProcessError> {
    let result = unsafe {
        libc::syscall(
            libc::SYS_pidfd_send_signal,
            fd,
            signal,
            std::ptr::null::<libc::siginfo_t>(),
            0,
        )
    };
    (result == 0)
        .then_some(())
        .ok_or(OwnedProcessError::Admission)
}

fn wait_for_recovery(pid: u32) -> Result<Option<()>, OwnedProcessError> {
    let mut status = 0;
    let result = unsafe { libc::waitpid(pid as libc::pid_t, &mut status, libc::WNOHANG) };
    if result == pid as libc::pid_t {
        Ok(Some(()))
    } else if result == 0 {
        Ok(None)
    } else {
        Err(OwnedProcessError::Admission)
    }
}

fn wait_for_recovery_blocking(pid: u32) -> Result<(), OwnedProcessError> {
    let mut status = 0;
    (unsafe { libc::waitpid(pid as libc::pid_t, &mut status, 0) } == pid as libc::pid_t)
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
