use serde::Serialize;
use tauri::ipc::Channel;

use super::app_update_assets::{
    current_architecture, current_platform, expected_asset_name, temp_extension,
    UpdateArchitecture, UpdatePlatform,
};
use super::app_update_download::{await_or_cancel, write_response_to_temporary};
use super::app_update_helper_process::spawn_update_helper;
use super::app_update_manifest::fetch_update_manifest;
use super::app_update_source::{
    download_client, strict_version_gt, update_request, AssetReference, UPDATE_SOURCE,
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
) -> Result<(), String> {
    let runtime = updates.inner().clone();
    let task_runtime = runtime.clone();
    runtime
        .run_download(move |cancellation| async move {
            download_app_update_inner(app, asset_url, on_progress, task_runtime, cancellation).await
        })
        .await
}

#[tauri::command]
pub fn cancel_app_update_download(
    updates: tauri::State<'_, crate::services::update_handoff::AppUpdateRuntime>,
) -> Result<(), String> {
    // Idempotent : un clic arrivé juste après la fin n'est pas une erreur.
    updates.cancel_active_download();
    Ok(())
}

async fn download_app_update_inner(
    app: tauri::AppHandle,
    asset_url: String,
    on_progress: Channel<DownloadProgress>,
    updates: crate::services::update_handoff::AppUpdateRuntime,
    cancellation: crate::services::work_registry::ServiceWorkCancellation,
) -> Result<(), String> {
    let asset = validate_update_url(&asset_url)?;
    let client = download_client().map_err(|_| download_error())?;
    let manifest_url = UPDATE_SOURCE
        .manifest_url(&asset.version)
        .ok_or_else(download_error)?;
    let manifest = await_or_cancel(
        &cancellation,
        fetch_update_manifest(&client, &manifest_url, &asset.version),
    )
    .await?
    .ok_or_else(download_error)?;
    let expected = manifest
        .asset_named(&asset.name)
        .ok_or_else(download_error)?;
    let response = await_or_cancel(
        &cancellation,
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
    let tmp = write_response_to_temporary(response, expected, ext, &cancellation, |progress| {
        let _ = on_progress.send(progress);
    })
    .await?;
    let helper = spawn_update_helper(&app, tmp.path(), &cancellation).await?;
    helper.commit(updates.handoff(), &cancellation)?;
    let _ = tmp.persist();
    crate::app_exit::request(&app, 0);

    Ok(())
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
