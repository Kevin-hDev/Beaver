use std::fmt;

use serde::de::{IgnoredAny, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer};
use subtle::ConstantTimeEq;

use super::app_update_source::{
    is_safe_asset_name, is_safe_version, update_request, UPDATE_SOURCE,
};
use crate::services::secure_http::read_bounded;

pub(crate) const MAX_UPDATE_MANIFEST_BYTES: usize = 64 * 1024;
pub(crate) const MAX_UPDATE_MANIFEST_ASSETS: usize = 16;
pub(crate) const MAX_UPDATE_ASSET_BYTES: u64 = 2 * 1024 * 1024 * 1024;
const UPDATE_MANIFEST_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(30);

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ManifestAsset {
    pub name: String,
    pub sha256: String,
    pub size: u64,
}

#[derive(Debug)]
pub(crate) struct UpdateManifest {
    assets: Vec<ManifestAsset>,
}

impl UpdateManifest {
    pub(crate) fn asset(&self, name: &str, size: u64) -> Option<&ManifestAsset> {
        self.assets
            .iter()
            .find(|asset| asset.name == name && asset.size == size)
    }

    pub(crate) fn asset_named(&self, name: &str) -> Option<&ManifestAsset> {
        self.assets.iter().find(|asset| asset.name == name)
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct UpdateManifestWire {
    version: String,
    #[serde(deserialize_with = "deserialize_manifest_assets")]
    assets: Vec<ManifestAsset>,
}

pub(crate) fn parse_update_manifest(
    bytes: &[u8],
    expected_version: &str,
) -> Option<UpdateManifest> {
    if bytes.is_empty()
        || bytes.len() > MAX_UPDATE_MANIFEST_BYTES
        || !is_safe_version(expected_version)
    {
        return None;
    }
    let wire: UpdateManifestWire = serde_json::from_slice(bytes).ok()?;
    if wire.version != expected_version || wire.assets.is_empty() {
        return None;
    }
    let prefix = format!("{}{expected_version}_", UPDATE_SOURCE.asset_prefix);
    for (index, asset) in wire.assets.iter().enumerate() {
        if !is_safe_asset_name(&asset.name)
            || !asset.name.starts_with(&prefix)
            || !valid_sha256(&asset.sha256)
            || asset.size == 0
            || asset.size > MAX_UPDATE_ASSET_BYTES
            || wire.assets[..index]
                .iter()
                .any(|previous| previous.name == asset.name)
        {
            return None;
        }
    }
    Some(UpdateManifest {
        assets: wire.assets,
    })
}

pub(crate) async fn fetch_update_manifest(
    client: &reqwest::Client,
    url: &url::Url,
    version: &str,
) -> Option<UpdateManifest> {
    let trusted_url = UPDATE_SOURCE.exact_manifest_url(url.as_str(), version)?;
    let body = tokio::time::timeout(UPDATE_MANIFEST_TIMEOUT, async {
        let response = update_request(client.get(trusted_url))
            .header(reqwest::header::ACCEPT_ENCODING, "identity")
            .send()
            .await
            .ok()?;
        if !response.status().is_success() {
            return None;
        }
        read_bounded(response, MAX_UPDATE_MANIFEST_BYTES).await.ok()
    })
    .await
    .ok()??;
    parse_update_manifest(&body, version)
}

pub(crate) fn sha256_matches(actual: &[u8; 32], expected_hex: &str) -> bool {
    if !valid_sha256(expected_hex) {
        return false;
    }
    let mut expected = [0_u8; 32];
    if hex::decode_to_slice(expected_hex, &mut expected).is_err() {
        return false;
    }
    actual.as_slice().ct_eq(expected.as_slice()).into()
}

pub(crate) fn checked_download_size(
    current: u64,
    chunk_bytes: usize,
    expected: u64,
) -> Option<u64> {
    if expected == 0 || expected > MAX_UPDATE_ASSET_BYTES {
        return None;
    }
    let next = current.checked_add(u64::try_from(chunk_bytes).ok()?)?;
    (next <= expected && next <= MAX_UPDATE_ASSET_BYTES).then_some(next)
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn deserialize_manifest_assets<'de, D>(deserializer: D) -> Result<Vec<ManifestAsset>, D::Error>
where
    D: Deserializer<'de>,
{
    deserializer.deserialize_seq(ManifestAssetsVisitor)
}

struct ManifestAssetsVisitor;

impl<'de> Visitor<'de> for ManifestAssetsVisitor {
    type Value = Vec<ManifestAsset>;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("une liste bornée d’assets de mise à jour")
    }

    fn visit_seq<A>(self, mut sequence: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let capacity = sequence
            .size_hint()
            .unwrap_or(0)
            .min(MAX_UPDATE_MANIFEST_ASSETS);
        let mut assets = Vec::with_capacity(capacity);
        while assets.len() < MAX_UPDATE_MANIFEST_ASSETS {
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

#[cfg(test)]
#[path = "app_update_manifest_tests.rs"]
mod tests;
