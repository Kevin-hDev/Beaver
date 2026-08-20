/// Arrête uniquement les descendants qui peuvent encore détenir les pipes
/// d'un parent déjà récolté. La racine n'est jamais resignalée : son PID
/// peut avoir été réutilisé entre `wait` et ce nettoyage.
pub(crate) fn kill_pipe_holders_after_parent_exit(root_pid: u32, kind: super::ProcessKind) {
    if root_pid < 2 {
        return;
    }
    let deadline = std::time::Instant::now() + super::GRACEFUL_STOP_TIMEOUT;
    #[cfg(unix)]
    terminate_group(root_pid, deadline);
    #[cfg(windows)]
    super::windows::terminate_descendants(root_pid, deadline);
    crate::services::owned_process::release(root_pid);
    ::log::info!(
        "[{}] descendants à pipes hérités arrêtés racine={root_pid}",
        kind.label()
    );
}

#[cfg(unix)]
fn terminate_group(process_group: u32, deadline: std::time::Instant) {
    signal_group(process_group, libc::SIGTERM);
    let remaining = deadline.saturating_duration_since(std::time::Instant::now());
    std::thread::sleep(remaining.min(std::time::Duration::from_millis(100)));
    signal_group(process_group, libc::SIGKILL);
}

#[cfg(unix)]
fn signal_group(process_group: u32, signal: libc::c_int) {
    let Ok(raw_group) = i32::try_from(process_group) else {
        return;
    };
    // SAFETY: un identifiant positif est validé par l'appelant et le PID de
    // la racine n'est pas ciblé ; seul le groupe créé au spawn l'est.
    unsafe {
        libc::kill(-raw_group, signal);
    }
}
