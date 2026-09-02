use super::ProcessKind;

pub async fn terminate_tokio_scoped(
    child: &mut tokio::process::Child,
    kind: ProcessKind,
    scope: &crate::services::owned_process::OwnedProcessScope,
    root_pid: u32,
    deadline: std::time::Instant,
) -> bool {
    #[cfg(windows)]
    let scope_terminated = super::windows::terminate_scope(scope);
    #[cfg(not(windows))]
    let scope_terminated = scope.terminate();
    let child_reaped = tokio::time::timeout_at(
        tokio::time::Instant::from_std(deadline),
        super::tokio_process::terminate_tokio_inner(child, kind, false),
    )
    .await
    .unwrap_or(false);
    let scope_empty = confirm_scope_absent(scope, root_pid, deadline).await;
    let confirmed = child_reaped && scope_empty;
    if confirmed {
        crate::services::owned_process::release(root_pid);
    } else if !scope_terminated {
        ::log::warn!("[process] terminaison du périmètre non confirmée");
    }
    confirmed
}

pub(super) async fn confirm_scope_absent(
    scope: &crate::services::owned_process::OwnedProcessScope,
    root_pid: u32,
    deadline: std::time::Instant,
) -> bool {
    #[cfg(unix)]
    return confirm_scope_absent_with_root_probe(
        scope,
        root_pid,
        deadline,
        crate::services::owned_process::OwnedProcess::process_exists,
    )
    .await;
    #[cfg(windows)]
    {
        // Windows confirme le Job Object complet ; le PID racine n'est utile
        // qu'au contrôle supplémentaire des groupes de processus Unix.
        let _ = root_pid;
        loop {
            let empty = super::windows::scope_is_empty(scope);
            if empty {
                return true;
            }
            if std::time::Instant::now() >= deadline {
                return false;
            }
            tokio::time::sleep(super::POLL_INTERVAL).await;
        }
    }
}

#[cfg(unix)]
pub(super) async fn confirm_scope_absent_with_root_probe(
    scope: &crate::services::owned_process::OwnedProcessScope,
    root_pid: u32,
    deadline: std::time::Instant,
    root_exists: impl Fn(u32) -> bool,
) -> bool {
    let _ = scope;
    // Après récolte du parent, un PID racine de nouveau présent appartient à
    // un autre processus : son numéro de groupe ne doit jamais être signalé.
    if root_exists(root_pid) {
        return false;
    }
    if std::time::Instant::now() < deadline {
        let _ = super::unix::terminate_group_members(root_pid, deadline).await;
    }
    loop {
        if super::unix::group_is_empty(root_pid) {
            return true;
        }
        if std::time::Instant::now() >= deadline {
            return false;
        }
        tokio::time::sleep(super::POLL_INTERVAL).await;
    }
}
