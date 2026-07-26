use serde::{Deserialize, Serialize};

use super::app_update_assets::{
    deserialize_release_assets, find_release_asset, find_release_manifest, ReleaseAsset,
    UpdateArchitecture, UpdatePlatform,
};
use super::app_update_manifest::MAX_UPDATE_ASSET_BYTES;
use super::app_update_notes::AppReleaseNotesByLocale;
use super::app_update_source::{strict_version_gt, MAX_RELEASE_RESPONSE_BYTES, UPDATE_SOURCE};

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

pub(crate) fn app_update_from_json(
    bytes: &[u8],
    current: &str,
    platform: UpdatePlatform,
    architecture: UpdateArchitecture,
) -> Option<AppUpdateInfo> {
    if bytes.len() > MAX_RELEASE_RESPONSE_BYTES {
        return None;
    }
    let release: GithubRelease = serde_json::from_slice(bytes).ok()?;
    app_update_from_release(&release, current, platform, architecture)
}

fn app_update_from_release(
    release: &GithubRelease,
    current: &str,
    platform: UpdatePlatform,
    architecture: UpdateArchitecture,
) -> Option<AppUpdateInfo> {
    if release.draft || release.prerelease {
        return None;
    }
    let version = release.tag_name.strip_prefix('v')?;
    if !strict_version_gt(version, current) {
        return None;
    }
    let expected_title = format!("{} v{version}", UPDATE_SOURCE.release_product);
    if release.name != expected_title {
        return None;
    }
    let asset = find_release_asset(
        &release.assets,
        &UPDATE_SOURCE,
        version,
        platform,
        architecture,
    )?;
    if asset.size == 0 || asset.size > MAX_UPDATE_ASSET_BYTES {
        return None;
    }
    let manifest = find_release_manifest(&release.assets, &UPDATE_SOURCE, version)?;
    Some(AppUpdateInfo {
        version: version.to_string(),
        asset_url: asset.url,
        title: Some(expected_title),
        published_at: validated_timestamp(release.published_at.as_deref())?,
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
