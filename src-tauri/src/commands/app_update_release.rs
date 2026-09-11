use serde::{Deserialize, Serialize};

use super::app_update_assets::{
    deserialize_release_assets, find_release_asset, find_release_manifest, ReleaseAsset,
    UpdateArchitecture, UpdatePlatform,
};
use super::app_update_manifest::MAX_UPDATE_ASSET_BYTES;
use super::app_update_notes::AppReleaseNotesByLocale;
use super::app_update_source::{
    is_safe_version, strict_version_gt, MAX_RELEASE_RESPONSE_BYTES, UPDATE_SOURCE,
};

pub(crate) enum AppUpdateClassification {
    NotNewer,
    Available(AppUpdateInfo),
    Invalid,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AppUpdateInfo {
    pub version: String,
    pub asset_url: String,
    pub title: Option<String>,
    pub published_at: Option<String>,
    pub notes_by_locale: Option<AppReleaseNotesByLocale>,
    #[serde(skip)]
    pub(crate) asset_name: String,
    #[serde(skip)]
    pub(crate) asset_size: u64,
    #[serde(skip)]
    pub(crate) manifest_url: String,
}

#[derive(Deserialize)]
struct GithubRelease {
    tag_name: String,
    name: String,
    published_at: Option<String>,
    draft: bool,
    prerelease: bool,
    #[serde(deserialize_with = "deserialize_release_assets")]
    assets: Vec<ReleaseAsset>,
}

#[cfg(test)]
pub(crate) fn app_update_from_json(
    bytes: &[u8],
    current: &str,
    platform: UpdatePlatform,
    architecture: UpdateArchitecture,
) -> Option<AppUpdateInfo> {
    match classify_app_update_json(bytes, current, platform, architecture) {
        AppUpdateClassification::Available(update) => Some(update),
        AppUpdateClassification::NotNewer | AppUpdateClassification::Invalid => None,
    }
}

pub(crate) fn classify_app_update_json(
    bytes: &[u8],
    current: &str,
    platform: UpdatePlatform,
    architecture: UpdateArchitecture,
) -> AppUpdateClassification {
    if bytes.len() > MAX_RELEASE_RESPONSE_BYTES {
        return AppUpdateClassification::Invalid;
    }
    let Ok(release) = serde_json::from_slice::<GithubRelease>(bytes) else {
        return AppUpdateClassification::Invalid;
    };
    app_update_from_release(&release, current, platform, architecture)
}

fn app_update_from_release(
    release: &GithubRelease,
    current: &str,
    platform: UpdatePlatform,
    architecture: UpdateArchitecture,
) -> AppUpdateClassification {
    if release.draft || release.prerelease {
        return AppUpdateClassification::Invalid;
    }
    let Some(version) = release.tag_name.strip_prefix('v') else {
        return AppUpdateClassification::Invalid;
    };
    if !is_safe_version(version) {
        return AppUpdateClassification::Invalid;
    }
    if !strict_version_gt(version, current) {
        return AppUpdateClassification::NotNewer;
    }
    let expected_title = format!("{} v{version}", UPDATE_SOURCE.release_product);
    if release.name != expected_title {
        return AppUpdateClassification::Invalid;
    }
    let Some(asset) = find_release_asset(
        &release.assets,
        &UPDATE_SOURCE,
        version,
        platform,
        architecture,
    ) else {
        return AppUpdateClassification::Invalid;
    };
    if asset.size == 0 || asset.size > MAX_UPDATE_ASSET_BYTES {
        return AppUpdateClassification::Invalid;
    }
    let Some(manifest) = find_release_manifest(&release.assets, &UPDATE_SOURCE, version) else {
        return AppUpdateClassification::Invalid;
    };
    let Some(published_at) = validated_timestamp(release.published_at.as_deref()) else {
        return AppUpdateClassification::Invalid;
    };
    AppUpdateClassification::Available(AppUpdateInfo {
        version: version.to_string(),
        asset_url: asset.url,
        title: Some(expected_title),
        published_at,
        notes_by_locale: None,
        asset_name: asset.name,
        asset_size: asset.size,
        manifest_url: manifest.url,
    })
}

fn validated_timestamp(value: Option<&str>) -> Option<Option<String>> {
    match value {
        None => Some(None),
        Some(value)
            if value.len() <= 64
                && value.trim() == value
                && chrono::DateTime::parse_from_rfc3339(value).is_ok() =>
        {
            Some(Some(value.to_string()))
        }
        Some(_) => None,
    }
}
