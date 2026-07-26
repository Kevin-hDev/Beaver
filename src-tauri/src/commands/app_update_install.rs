use serde::Serialize;
use sha2::{Digest, Sha256};
use tauri::ipc::Channel;

use super::app_update_assets::{
    current_architecture, current_platform, expected_asset_name, temp_extension,
    UpdateArchitecture, UpdatePlatform,
};
use super::app_update_helper::spawn_update_helper;
use super::app_update_install_temp::create_unique_temp_file;
use super::app_update_manifest::{checked_download_size, fetch_update_manifest, sha256_matches};
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
) -> Result<(), String> {
    let asset = validate_update_url(&asset_url)?;
    let client = download_client().map_err(|_| download_error())?;
    let manifest_url = UPDATE_SOURCE
        .manifest_url(&asset.version)
        .ok_or_else(download_error)?;
    let manifest = fetch_update_manifest(&client, &manifest_url, &asset.version)
        .await
        .ok_or_else(download_error)?;
    let expected = manifest
        .asset_named(&asset.name)
        .ok_or_else(download_error)?;
    let resp = update_request(client.get(asset.url))
        .header(reqwest::header::ACCEPT_ENCODING, "identity")
        .send()
        .await
        .map_err(|_| download_error())?;

    if !resp.status().is_success() {
        return Err(download_error());
    }

    if resp
        .content_length()
        .is_some_and(|length| length != expected.size)
    {
        return Err(download_error());
    }
    let total = expected.size;
    let ext = temp_extension(current_platform());
    let (tmp, file) = create_unique_temp_file(
        crate::updater_worker::UPDATE_TEMP_PREFIX,
        &format!(".{ext}"),
    )?;
    let mut file = tokio::fs::File::from_std(file);

    use futures_util::StreamExt;
    use tokio::io::AsyncWriteExt;

    let mut stream = resp.bytes_stream();
    let mut downloaded = 0_u64;
    let mut hasher = Sha256::new();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|_| download_error())?;
        let next = checked_download_size(downloaded, chunk.len(), expected.size)
            .ok_or_else(download_error)?;
        hasher.update(&chunk);
        file.write_all(&chunk).await.map_err(|_| write_error())?;
        downloaded = next;
        let _ = on_progress.send(DownloadProgress {
            completed: downloaded,
            total,
        });
    }

    let actual: [u8; 32] = hasher.finalize().into();
    if downloaded != expected.size || !sha256_matches(&actual, &expected.sha256) {
        return Err(download_error());
    }
    file.flush().await.map_err(|_| write_error())?;
    file.sync_all().await.map_err(|_| write_error())?;
    drop(file);

    spawn_update_helper(&app, tmp.path())?;
    let _ = tmp.persist();
    app.exit(0);

    Ok(())
}

fn update_url_error() -> String {
    "update-url-invalid".to_string()
}

fn download_error() -> String {
    "update-download-error".to_string()
}

fn write_error() -> String {
    "update-write-error".to_string()
}

#[cfg(test)]
#[path = "app_update_install_tests.rs"]
mod tests;
