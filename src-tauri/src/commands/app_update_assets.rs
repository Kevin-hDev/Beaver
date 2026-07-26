use std::fmt;

use serde::de::{IgnoredAny, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer};

use super::app_update_manifest::MAX_UPDATE_MANIFEST_BYTES;
use super::app_update_source::UPDATE_MANIFEST_NAME;
use super::app_update_source::{is_safe_version, UpdateSource};

pub(crate) const MAX_RELEASE_ASSETS: usize = 64;
const MAX_ASSET_NAME_BYTES: usize = 128;

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct ReleaseAsset {
    pub name: String,
    pub browser_download_url: String,
    pub size: u64,
}

#[derive(Debug)]
pub(crate) struct SelectedReleaseAsset {
    pub name: String,
    pub url: String,
    pub size: u64,
}

pub(crate) fn deserialize_release_assets<'de, D>(
    deserializer: D,
) -> Result<Vec<ReleaseAsset>, D::Error>
where
    D: Deserializer<'de>,
{
    deserializer.deserialize_seq(ReleaseAssetsVisitor)
}

struct ReleaseAssetsVisitor;

impl<'de> Visitor<'de> for ReleaseAssetsVisitor {
    type Value = Vec<ReleaseAsset>;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("une liste bornée d’assets")
    }

    fn visit_seq<A>(self, mut sequence: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let capacity = sequence.size_hint().unwrap_or(0).min(MAX_RELEASE_ASSETS);
        let mut assets = Vec::with_capacity(capacity);
        while assets.len() < MAX_RELEASE_ASSETS {
            let Some(asset) = sequence.next_element()? else {
                return Ok(assets);
            };
            assets.push(asset);
        }
        if sequence.next_element::<IgnoredAny>()?.is_some() {
            return Err(serde::de::Error::custom("trop d’assets"));
        }
        Ok(assets)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum UpdatePlatform {
    Macos,
    Windows,
    Linux,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum UpdateArchitecture {
    Aarch64,
    X86_64,
}

pub(crate) fn current_platform() -> UpdatePlatform {
    if cfg!(target_os = "macos") {
        UpdatePlatform::Macos
    } else if cfg!(target_os = "windows") {
        UpdatePlatform::Windows
    } else {
        UpdatePlatform::Linux
    }
}

pub(crate) fn current_architecture() -> Option<UpdateArchitecture> {
    if cfg!(target_arch = "aarch64") {
        Some(UpdateArchitecture::Aarch64)
    } else if cfg!(target_arch = "x86_64") {
        Some(UpdateArchitecture::X86_64)
    } else {
        None
    }
}

pub(crate) fn temp_extension(platform: UpdatePlatform) -> &'static str {
    match platform {
        UpdatePlatform::Macos => "dmg",
        UpdatePlatform::Windows => "exe",
        UpdatePlatform::Linux => "deb",
    }
}

fn asset_suffix(platform: UpdatePlatform, architecture: UpdateArchitecture) -> &'static str {
    match (platform, architecture) {
        (UpdatePlatform::Macos, UpdateArchitecture::Aarch64) => "_aarch64.dmg",
        (UpdatePlatform::Macos, UpdateArchitecture::X86_64) => "_x64.dmg",
        (UpdatePlatform::Windows, UpdateArchitecture::Aarch64) => "_arm64-setup.exe",
        (UpdatePlatform::Windows, UpdateArchitecture::X86_64) => "_x64-setup.exe",
        (UpdatePlatform::Linux, UpdateArchitecture::Aarch64) => "_arm64.deb",
        (UpdatePlatform::Linux, UpdateArchitecture::X86_64) => "_amd64.deb",
    }
}

pub(crate) fn expected_asset_name(
    source: &UpdateSource,
    version: &str,
    platform: UpdatePlatform,
    architecture: UpdateArchitecture,
) -> Option<String> {
    if !is_safe_version(version) {
        return None;
    }
    let name = format!(
        "{}{}{}",
        source.asset_prefix,
        version,
        asset_suffix(platform, architecture)
    );
    (name.len() <= MAX_ASSET_NAME_BYTES).then_some(name)
}

pub(crate) fn find_release_asset(
    assets: &[ReleaseAsset],
    source: &UpdateSource,
    version: &str,
    platform: UpdatePlatform,
    architecture: UpdateArchitecture,
) -> Option<SelectedReleaseAsset> {
    let expected = expected_asset_name(source, version, platform, architecture)?;
    let mut found = None;
    for asset in assets {
        if asset.name != expected {
            continue;
        }
        if found.is_some() {
            return None;
        }
        let url = source.exact_asset_url(&asset.browser_download_url, version, &expected)?;
        found = Some(SelectedReleaseAsset {
            name: asset.name.clone(),
            url: url.to_string(),
            size: asset.size,
        });
    }
    found
}

pub(crate) fn find_release_manifest(
    assets: &[ReleaseAsset],
    source: &UpdateSource,
    version: &str,
) -> Option<SelectedReleaseAsset> {
    let mut found = None;
    for asset in assets {
        if asset.name != UPDATE_MANIFEST_NAME {
            continue;
        }
        if found.is_some() || asset.size == 0 || asset.size > MAX_UPDATE_MANIFEST_BYTES as u64 {
            return None;
        }
        let url = source.exact_manifest_url(&asset.browser_download_url, version)?;
        found = Some(SelectedReleaseAsset {
            name: asset.name.clone(),
            url: url.to_string(),
            size: asset.size,
        });
    }
    found
}

#[cfg(test)]
#[path = "app_update_assets_tests.rs"]
mod tests;
