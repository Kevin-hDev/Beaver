use super::runtime::ExtensionRuntime;
use super::types::{ExtensionOriginKind, ExtensionRecord};
use super::OperationFailure;
use std::sync::Arc;

pub use super::installer_uninstall::uninstall;

pub(super) fn extension_runtime() -> Result<Arc<ExtensionRuntime>, OperationFailure> {
    super::runtime::global()
        .map(Arc::clone)
        .map_err(|_| OperationFailure::HostUnavailable)
}
pub(super) fn is_managed(record: &ExtensionRecord) -> bool {
    record.origin.as_ref().is_some_and(|origin| {
        matches!(
            origin.kind,
            ExtensionOriginKind::Git | ExtensionOriginKind::Npm
        )
    })
}

pub(super) async fn blocking<T: Send + 'static>(
    operation: impl FnOnce() -> Result<T, OperationFailure> + Send + 'static,
    interrupted: OperationFailure,
) -> Result<T, OperationFailure> {
    tokio::task::spawn_blocking(operation)
        .await
        .map_err(|_| interrupted)?
}
