use super::{ProcessKind, GRACEFUL_STOP_TIMEOUT, POLL_INTERVAL};

pub async fn terminate_tokio(child: &mut tokio::process::Child, kind: ProcessKind) {
    let _ = terminate_tokio_inner(child, kind, true).await;
}

pub(super) async fn terminate_tokio_inner(
    child: &mut tokio::process::Child,
    kind: ProcessKind,
    release_identity: bool,
) -> bool {
    if child.try_wait().ok().flatten().is_some() {
        if release_identity {
            if let Some(pid) = child.id() {
                crate::services::owned_process::release(pid);
            }
        }
        return true;
    }
    let Some(pid) = child.id() else {
        return child.try_wait().ok().flatten().is_some();
    };
    super::signal_tree(pid, false);
    let deadline = tokio::time::Instant::now() + GRACEFUL_STOP_TIMEOUT;
    while tokio::time::Instant::now() < deadline {
        if child.try_wait().ok().flatten().is_some() {
            if release_identity {
                crate::services::owned_process::release(pid);
            }
            ::log::info!("[{}] arbre pid={pid} arrêté", kind.label());
            return true;
        }
        tokio::time::sleep(POLL_INTERVAL).await;
    }
    super::force_tree(pid);
    let _ = child.start_kill();
    let _ = child.wait().await;
    if release_identity {
        crate::services::owned_process::release(pid);
    }
    ::log::warn!("[{}] arrêt forcé arbre pid={pid}", kind.label());
    child.try_wait().ok().flatten().is_some()
}
