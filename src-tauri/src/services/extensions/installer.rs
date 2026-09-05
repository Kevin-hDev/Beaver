//! Compatibility commands wait on the same owner as background installations.
use super::install_jobs::{self, InstallRequest};
use super::runtime::ExtensionRuntime;
use super::types::{ExtensionOriginKind, ExtensionRecord};
use super::OperationFailure;
use std::sync::Arc;
use std::time::Instant;

pub use super::installer_uninstall::uninstall;

pub async fn install_git(
    _app: &tauri::AppHandle,
    locator: &str,
    _deadline: Instant,
) -> Result<ExtensionRecord, OperationFailure> {
    install_jobs::global()
        .map_err(|_| OperationFailure::HostUnavailable)?
        .wait_install(InstallRequest::Git {
            locator: locator.into(),
        })
        .await
}
pub async fn install_npm(
    _app: &tauri::AppHandle,
    locator: &str,
    _deadline: Instant,
) -> Result<ExtensionRecord, OperationFailure> {
    install_jobs::global()
        .map_err(|_| OperationFailure::HostUnavailable)?
        .wait_install(InstallRequest::Npm {
            locator: locator.into(),
        })
        .await
}
pub async fn update(
    _app: &tauri::AppHandle,
    id: &str,
    _deadline: Instant,
) -> Result<ExtensionRecord, OperationFailure> {
    install_jobs::global()
        .map_err(|_| OperationFailure::HostUnavailable)?
        .wait_install(InstallRequest::Update {
            extension_id: id.into(),
        })
        .await
}

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
