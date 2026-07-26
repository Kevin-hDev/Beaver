use reqwest::header::ACCEPT;

use super::app_update_assets::{current_architecture, current_platform};
use super::app_update_manifest::fetch_update_manifest;
use super::app_update_notes::{
    parse_app_release_notes_json, AppReleaseNotesByLocale, MAX_RELEASE_NOTES_BYTES,
};
use super::app_update_release::app_update_from_json;
pub use super::app_update_release::AppUpdateInfo;
use super::app_update_source::{
    download_client, metadata_client, update_request, MAX_RELEASE_RESPONSE_BYTES, UPDATE_SOURCE,
};
use crate::services::secure_http::read_bounded;

#[tauri::command]
pub async fn check_app_update() -> Result<Option<AppUpdateInfo>, String> {
    let architecture = match current_architecture() {
        Some(architecture) => architecture,
        None => return Ok(None),
    };
    let client = metadata_client().map_err(|_| update_error())?;
    let url = UPDATE_SOURCE
        .latest_release_url()
        .ok_or_else(update_error)?;
    let request = update_request(client.get(url)).header(ACCEPT, "application/vnd.github+json");
    let response = client.send(request).await.map_err(|_| update_error())?;
    if !response.status().is_success() {
        return Ok(None);
    }
    let body = read_bounded(response, MAX_RELEASE_RESPONSE_BYTES)
        .await
        .map_err(|_| update_error())?;
    let mut update = match app_update_from_json(
        &body,
        env!("CARGO_PKG_VERSION"),
        current_platform(),
        architecture,
    ) {
        Some(update) => update,
        None => return Ok(None),
    };
    let manifest_client = download_client().map_err(|_| update_error())?;
    let manifest_url = url::Url::parse(&update.manifest_url).map_err(|_| update_error())?;
    let Some(manifest) =
        fetch_update_manifest(&manifest_client, &manifest_url, &update.version).await
    else {
        return Ok(None);
    };
    if manifest
        .asset(&update.asset_name, update.asset_size)
        .is_none()
    {
        return Ok(None);
    }
    update.notes_by_locale = fetch_release_notes(&client, &update.version).await;
    Ok(Some(update))
}

async fn fetch_release_notes(
    client: &crate::services::secure_http::AuthenticatedClient,
    version: &str,
) -> Option<AppReleaseNotesByLocale> {
    let url = UPDATE_SOURCE.release_notes_url(version)?;
    let response = client.send(update_request(client.get(url))).await.ok()?;
    if !response.status().is_success() {
        return None;
    }
    let body = read_bounded(response, MAX_RELEASE_NOTES_BYTES).await.ok()?;
    parse_app_release_notes_json(&body, version)
}

fn update_error() -> String {
    "update-check-error".to_string()
}

pub(crate) fn version_gt(remote: &str, local: &str) -> bool {
    if remote.len() > 64 || local.len() > 64 {
        return false;
    }
    let mut remote_parts = remote.split('.');
    let mut local_parts = local.split('.');
    for _ in 0..8 {
        let remote = remote_parts.next();
        let local = local_parts.next();
        if remote.is_none() && local.is_none() {
            return false;
        }
        let remote = remote
            .and_then(|part| part.parse::<u64>().ok())
            .unwrap_or(0);
        let local = local.and_then(|part| part.parse::<u64>().ok()).unwrap_or(0);
        if remote != local {
            return remote > local;
        }
    }
    false
}

#[cfg(test)]
#[path = "app_update_limits_tests.rs"]
mod limits_tests;

#[cfg(test)]
#[path = "app_update_tests.rs"]
mod tests;
