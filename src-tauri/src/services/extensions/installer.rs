use super::install_preparation::PreparedInstall;
use super::runtime::ExtensionRuntime;
use super::types::{ExtensionOriginKind, ExtensionRecord};
use super::OperationFailure;
use crate::services::work_registry::ServiceWorkCancellation;
use std::sync::Arc;

pub async fn install_git(
    app: &tauri::AppHandle,
    locator: &str,
) -> Result<ExtensionRecord, OperationFailure> {
    let source =
        super::source_validation::git(locator).map_err(|_| OperationFailure::SourceInvalid)?;
    let npm = super::npm_runner::NpmRunner::resolve(app)?;
    let work = extension_runtime()?.work.clone();
    work.run_operation(move |cancel| async move {
        let prepared = blocking(
            move || super::install_preparation::git(source, npm, &cancel),
            OperationFailure::InstallFailed,
        )
        .await?;
        register_new(prepared)
    })
    .await
    .map_err(|error| error.operation_failure())?
}

pub async fn install_npm(
    app: &tauri::AppHandle,
    locator: &str,
) -> Result<ExtensionRecord, OperationFailure> {
    let source =
        super::source_validation::npm(locator).map_err(|_| OperationFailure::PackageInvalid)?;
    let npm = super::npm_runner::NpmRunner::resolve(app)?;
    let work = extension_runtime()?.work.clone();
    work.run_operation(move |cancel| async move {
        let prepared = blocking(
            move || super::install_preparation::npm(source, npm, &cancel),
            OperationFailure::InstallFailed,
        )
        .await?;
        register_new(prepared)
    })
    .await
    .map_err(|error| error.operation_failure())?
}

pub async fn update(app: &tauri::AppHandle, id: &str) -> Result<ExtensionRecord, OperationFailure> {
    let current = super::registry::find(id).map_err(|_| OperationFailure::UpdateUnavailable)?;
    let origin = current
        .origin
        .clone()
        .filter(|origin| is_managed_kind(&origin.kind))
        .ok_or(OperationFailure::UpdateUnavailable)?;
    let npm = super::npm_runner::NpmRunner::resolve(app)?;
    let runtime = extension_runtime()?;
    let work = runtime.work.clone();
    work.run_operation(move |cancel| async move {
        let prepared = prepare_update(origin, npm, cancel.clone()).await?;
        if prepared.record.manifest.id != current.manifest.id {
            cleanup(&prepared.record).await;
            return Err(OperationFailure::UpdateIdentityChanged);
        }
        replace_current(&runtime, current, prepared).await
    })
    .await
    .map_err(|error| error.operation_failure())?
}

pub async fn uninstall(id: &str) -> Result<(), OperationFailure> {
    let current = super::registry::find(id).map_err(|_| OperationFailure::UninstallFailed)?;
    let id = id.to_string();
    let runtime = extension_runtime()?;
    let work = runtime.work.clone();
    work.run_operation(move |_| async move {
        runtime.stop_host().await;
        if super::registry::remove(&id).is_err() {
            let _ = runtime.start_untracked().await;
            return Err(OperationFailure::UninstallFailed);
        }
        let result = if is_managed(&current) {
            let record = current.clone();
            blocking(
                move || {
                    super::managed_store::remove_record(&record)
                        .map_err(|_| OperationFailure::StorageFailed)
                },
                OperationFailure::UninstallFailed,
            )
            .await
        } else {
            Ok(())
        };
        let _ = runtime.start_untracked().await;
        result
    })
    .await
    .map_err(|error| error.operation_failure())?
}

async fn prepare_update(
    origin: super::types::ExtensionOrigin,
    npm: super::npm_runner::NpmRunner,
    cancellation: ServiceWorkCancellation,
) -> Result<PreparedInstall, OperationFailure> {
    match origin.kind {
        ExtensionOriginKind::Git => {
            let source = super::source_validation::git(&origin.locator)
                .map_err(|_| OperationFailure::SourceInvalid)?;
            blocking(
                move || super::install_preparation::git(source, npm, &cancellation),
                OperationFailure::UpdateFailed,
            )
            .await
        }
        ExtensionOriginKind::Npm => {
            let source = super::source_validation::npm(&origin.locator)
                .map_err(|_| OperationFailure::PackageInvalid)?;
            blocking(
                move || super::install_preparation::npm(source, npm, &cancellation),
                OperationFailure::UpdateFailed,
            )
            .await
        }
        ExtensionOriginKind::Local => Err(OperationFailure::UpdateUnavailable),
    }
}

fn register_new(prepared: PreparedInstall) -> Result<ExtensionRecord, OperationFailure> {
    let record = prepared.record;
    if let Err(error) = super::registry_managed::add(record.clone()) {
        let _ = super::managed_store::remove_record(&record);
        return Err(error);
    }
    Ok(record)
}

async fn replace_current(
    runtime: &ExtensionRuntime,
    current: ExtensionRecord,
    prepared: PreparedInstall,
) -> Result<ExtensionRecord, OperationFailure> {
    let replacement = super::installer_record::for_update(&current, prepared.record);
    runtime.stop_host().await;
    if super::registry::replace_user(&current, replacement.clone()).is_err() {
        cleanup(&replacement).await;
        let _ = runtime.start_untracked().await;
        return Err(OperationFailure::UpdateFailed);
    }
    let old = current.clone();
    let _ = blocking(
        move || {
            super::managed_store::remove_record(&old).map_err(|_| OperationFailure::StorageFailed)
        },
        OperationFailure::UpdateFailed,
    )
    .await;
    let _ = runtime.start_untracked().await;
    Ok(replacement)
}

async fn cleanup(record: &ExtensionRecord) {
    let record = record.clone();
    let _ = blocking(
        move || {
            super::managed_store::remove_record(&record)
                .map_err(|_| OperationFailure::StorageFailed)
        },
        OperationFailure::StorageFailed,
    )
    .await;
}

fn extension_runtime() -> Result<Arc<ExtensionRuntime>, OperationFailure> {
    super::runtime::global()
        .map(Arc::clone)
        .map_err(|_| OperationFailure::HostUnavailable)
}

fn is_managed(record: &ExtensionRecord) -> bool {
    record
        .origin
        .as_ref()
        .is_some_and(|origin| is_managed_kind(&origin.kind))
}

fn is_managed_kind(kind: &ExtensionOriginKind) -> bool {
    matches!(kind, ExtensionOriginKind::Git | ExtensionOriginKind::Npm)
}

async fn blocking<T: Send + 'static>(
    operation: impl FnOnce() -> Result<T, OperationFailure> + Send + 'static,
    interrupted: OperationFailure,
) -> Result<T, OperationFailure> {
    tokio::task::spawn_blocking(operation)
        .await
        .map_err(|_| interrupted)?
}
