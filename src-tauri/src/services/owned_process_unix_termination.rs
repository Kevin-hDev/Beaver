use super::{
    identity, process_exists, release, signal_exact, OwnedProcessError, OwnedProcessIdentity,
};

pub(in crate::services::owned_process) fn recover_exact_with_cancel(
    expected: OwnedProcessIdentity,
    deadline: std::time::Instant,
    cancelled: &dyn Fn() -> bool,
) -> Result<(), OwnedProcessError> {
    if identity(expected.pid)? != expected {
        return Err(OwnedProcessError::Admission);
    }
    signal_exact(expected, false)?;
    // Le délai fourni par l'appelant est l'unique budget : sa première moitié
    // est gracieuse et la seconde reste disponible pour la terminaison forcée.
    let now = std::time::Instant::now();
    let graceful_deadline = now + deadline.saturating_duration_since(now) / 2;
    if wait_for_object_exit(expected.pid, graceful_deadline, cancelled)? {
        return Ok(());
    }
    signal_exact(expected, true)?;
    wait_for_object_exit(expected.pid, deadline, cancelled)?
        .then_some(())
        .ok_or(OwnedProcessError::Admission)
}

fn wait_for_object_exit(
    pid: u32,
    deadline: std::time::Instant,
    cancelled: &dyn Fn() -> bool,
) -> Result<bool, OwnedProcessError> {
    let mut status = 0;
    while std::time::Instant::now() < deadline {
        if cancelled() {
            return Err(OwnedProcessError::Admission);
        }
        let result = unsafe { libc::waitpid(pid as libc::pid_t, &mut status, libc::WNOHANG) };
        if result == pid as libc::pid_t {
            release(pid);
            return Ok(true);
        }
        if result < 0 && std::io::Error::last_os_error().raw_os_error() != Some(libc::ECHILD) {
            return Err(OwnedProcessError::Admission);
        }
        if !process_exists(pid) {
            release(pid);
            return Ok(true);
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    Ok(false)
}
