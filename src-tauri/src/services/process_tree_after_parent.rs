/// Arrête uniquement les membres encore vérifiés du groupe d'un parent récolté.
/// Le numéro du groupe ne suffit jamais : chaque PID est lié à son heure de
/// démarrage puis revérifié juste avant le signal.
pub(crate) fn kill_pipe_holders_after_parent_exit(root_pid: u32, kind: super::ProcessKind) -> bool {
    kill_pipe_holders_with(
        root_pid,
        kind,
        crate::services::owned_process::OwnedProcess::process_exists,
        |root_pid, deadline| {
            #[cfg(unix)]
            {
                terminate_group(root_pid, deadline)
            }
            #[cfg(windows)]
            {
                super::windows::terminate_descendants(root_pid, deadline)
            }
        },
        crate::services::owned_process::release,
    )
}

pub(super) fn kill_pipe_holders_with<Exists, Terminate, Release>(
    root_pid: u32,
    kind: super::ProcessKind,
    process_exists: Exists,
    mut terminate: Terminate,
    release: Release,
) -> bool
where
    Exists: Fn(u32) -> bool,
    Terminate: FnMut(u32, std::time::Instant) -> bool,
    Release: Fn(u32),
{
    // `wait` a déjà récolté la racine. Si son PID existe encore, il a été
    // réutilisé : le numéro de groupe ne permet alors plus de relier ses
    // membres à l'ancien processus et aucun signal n'est sûr.
    if root_pid < 2 || process_exists(root_pid) {
        return false;
    }
    let deadline = std::time::Instant::now() + super::GRACEFUL_STOP_TIMEOUT;
    let complete = terminate(root_pid, deadline);
    release(root_pid);
    if complete {
        ::log::info!(
            "[{}] descendants à pipes hérités arrêtés racine={root_pid}",
            kind.label()
        );
    } else {
        ::log::warn!(
            "[{}] nettoyage des descendants non confirmé racine={root_pid}",
            kind.label()
        );
    }
    complete
}

#[cfg(unix)]
fn terminate_group(process_group: u32, deadline: std::time::Instant) -> bool {
    let (members, complete) = super::unix::collect_group_members(process_group);
    if members.is_empty() {
        return false;
    }
    signal_members(&members, process_group, libc::SIGTERM);
    let remaining = deadline.saturating_duration_since(std::time::Instant::now());
    std::thread::sleep(remaining.min(std::time::Duration::from_millis(100)));
    signal_members(&members, process_group, libc::SIGKILL);
    complete
        && members
            .into_iter()
            .all(|member| !super::unix::is_current_group_member(member, process_group))
}

#[cfg(unix)]
fn signal_members(
    members: &[super::unix::UnixProcessIdentity],
    process_group: u32,
    signal: libc::c_int,
) {
    signal_members_with(
        members,
        process_group,
        signal,
        super::unix::is_current_group_member,
        |pid, signal| unsafe {
            libc::kill(pid, signal);
        },
    );
}

#[cfg(unix)]
pub(super) fn signal_members_with<Current, Signal>(
    members: &[super::unix::UnixProcessIdentity],
    process_group: u32,
    signal: libc::c_int,
    is_current: Current,
    mut send_signal: Signal,
) where
    Current: Fn(super::unix::UnixProcessIdentity, u32) -> bool,
    Signal: FnMut(i32, libc::c_int),
{
    for member in members.iter().copied() {
        if !is_current(member, process_group) {
            continue;
        }
        let Ok(pid) = i32::try_from(member.pid().as_u32()) else {
            continue;
        };
        send_signal(pid, signal);
    }
}
