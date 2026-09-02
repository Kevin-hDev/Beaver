use super::ProcessKind;

pub async fn terminate_tokio_scoped(
    child: &mut tokio::process::Child,
    kind: ProcessKind,
    scope: &crate::services::owned_process::OwnedProcessScope,
) -> bool {
    #[cfg(windows)]
    let scope_terminated = super::windows::terminate_scope(scope);
    #[cfg(not(windows))]
    let scope_terminated = scope.terminate();
    super::terminate_tokio(child, kind).await;
    scope_terminated
}
