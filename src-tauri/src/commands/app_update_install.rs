use serde::Serialize;
use tauri::ipc::Channel;

use super::app_update_assets::{
    current_architecture, current_platform, expected_asset_name, temp_extension,
    UpdateArchitecture, UpdatePlatform,
};
use super::app_update_download::{await_or_cancel, write_response_to_temporary};
use super::app_update_install_temp::TemporaryUpdate;
use super::app_update_manifest::fetch_update_manifest;
use super::app_update_progress::{finish as finish_progress, running_operation, terminal_status};
use super::app_update_source::{
    download_client, strict_version_gt, update_request, AssetReference, UPDATE_SOURCE,
};
use crate::services::update_progress::{
    UpdateOperationKind, UpdateOperationPhase, UpdateOperationStatus, UpdateProgressMode,
    UpdateProgressRuntime,
};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadProgress {
    pub completed: u64,
    pub total: u64,
}

fn validate_update_url_for(
    raw: &str,
    platform: UpdatePlatform,
    architecture: UpdateArchitecture,
) -> Result<AssetReference, String> {
    let reference = UPDATE_SOURCE
        .asset_reference(raw)
        .ok_or_else(update_url_error)?;
    if !strict_version_gt(&reference.version, env!("CARGO_PKG_VERSION")) {
        return Err(update_url_error());
    }
    let expected = expected_asset_name(&UPDATE_SOURCE, &reference.version, platform, architecture)
        .ok_or_else(update_url_error)?;
    if reference.name != expected {
        return Err(update_url_error());
    }
    Ok(reference)
}

fn validate_update_url(raw: &str) -> Result<AssetReference, String> {
    let architecture = current_architecture().ok_or_else(update_url_error)?;
    validate_update_url_for(raw, current_platform(), architecture)
}

#[tauri::command]
pub async fn download_app_update(
    app: tauri::AppHandle,
    asset_url: String,
    on_progress: Channel<DownloadProgress>,
    updates: tauri::State<'_, crate::services::update_handoff::AppUpdateRuntime>,
    progress: tauri::State<'_, UpdateProgressRuntime>,
) -> Result<(), String> {
    let asset = validate_update_url(&asset_url)?;
    let id = uuid::Uuid::new_v4().to_string();
    let label = format!("Beaver {}", asset.version);
    progress.upsert(
        &app,
        running_operation(
            &id,
            &label,
            UpdateOperationPhase::Preparing,
            UpdateProgressMode::Indeterminate,
            None,
            true,
        ),
    )?;
    let runtime = updates.inner().clone();
    let task_runtime = runtime.clone();
    let task_progress = progress.inner().clone();
    let finish_app = app.clone();
    let finish_progress_runtime = progress.inner().clone();
    let finish_id = id.clone();
    let result = runtime
        .run_download(move |cancellation| async move {
            super::app_update_install_flow::run(
                app,
                asset,
                on_progress,
                task_runtime,
                task_progress,
                (id, label),
                cancellation,
            )
            .await
        })
        .await;
    if let Err(error) = &result {
        let (status, error_key) = terminal_status(Some(error));
        finish_progress(
            &finish_app,
            &finish_progress_runtime,
            &finish_id,
            status,
            error_key,
        )?;
    }
    result
}

#[tauri::command]
pub fn cancel_app_update_download(
    app: tauri::AppHandle,
    updates: tauri::State<'_, crate::services::update_handoff::AppUpdateRuntime>,
    progress: tauri::State<'_, UpdateProgressRuntime>,
) -> Result<(), String> {
    // Idempotent : un clic arrivé juste après la fin n'est pas une erreur.
    let cancellable = progress.snapshot()?.into_iter().any(|operation| {
        operation.kind == UpdateOperationKind::AppRelease
            && !operation.status.is_terminal()
            && operation.can_cancel
    });
    if cancellable && updates.cancel_active_download() {
        for mut operation in progress.snapshot()? {
            if operation.kind == UpdateOperationKind::AppRelease && !operation.status.is_terminal()
            {
                operation.status = UpdateOperationStatus::Cancelling;
                operation.can_cancel = false;
                progress.upsert(&app, operation)?;
            }
        }
    }
    Ok(())
}

pub(super) async fn download_verified_update<Progress>(
    asset_url: &str,
    cancellation: &crate::services::work_registry::ServiceWorkCancellation,
    progress: Progress,
) -> Result<TemporaryUpdate, String>
where
    Progress: FnMut(DownloadProgress),
{
    let asset = validate_update_url(asset_url)?;
    download_verified_asset(asset, cancellation, progress).await
}

pub(super) async fn download_verified_asset<Progress>(
    asset: AssetReference,
    cancellation: &crate::services::work_registry::ServiceWorkCancellation,
    progress: Progress,
) -> Result<TemporaryUpdate, String>
where
    Progress: FnMut(DownloadProgress),
{
    let client = download_client().map_err(|_| download_error())?;
    let manifest_url = UPDATE_SOURCE
        .manifest_url(&asset.version)
        .ok_or_else(download_error)?;
    let manifest = await_or_cancel(
        cancellation,
        fetch_update_manifest(&client, &manifest_url, &asset.version),
    )
    .await?
    .ok_or_else(download_error)?;
    let expected = manifest
        .asset_named(&asset.name)
        .ok_or_else(download_error)?;
    let response = await_or_cancel(
        cancellation,
        update_request(client.get(asset.url))
            .header(reqwest::header::ACCEPT_ENCODING, "identity")
            .send(),
    )
    .await?
    .map_err(|_| download_error())?;

    if !response.status().is_success() {
        return Err(download_error());
    }

    if response
        .content_length()
        .is_some_and(|length| length != expected.size)
    {
        return Err(download_error());
    }
    let ext = temp_extension(current_platform());
    write_response_to_temporary(response, expected, ext, cancellation, progress).await
}

fn update_url_error() -> String {
    "update-url-invalid".to_string()
}

fn download_error() -> String {
    "update-download-error".to_string()
}

#[cfg(test)]
#[path = "app_update_install_tests.rs"]
mod tests;
