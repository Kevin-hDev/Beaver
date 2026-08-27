use super::error::OllamaErrorCode;
use super::fingerprint::OllamaVersion;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

const CACHE_SCHEMA_VERSION: u8 = 1;
const MAX_CACHE_BYTES: u64 = 256;

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct CachedRelease {
    schema_version: u8,
    version: OllamaVersion,
}

pub(crate) async fn fetch_latest_version_for_update_check() -> Result<OllamaVersion, OllamaErrorCode>
{
    let remote = super::release_fetch::fetch_latest_version().await;
    // The update screen may reuse a confirmed release during a transient network failure.
    resolve_at_path(&cache_path(), remote)
}

fn cache_path() -> PathBuf {
    crate::services::paths::data_dir().join("ollama-release-cache.json")
}

pub(super) fn resolve_at_path(
    path: &Path,
    remote: Result<OllamaVersion, OllamaErrorCode>,
) -> Result<OllamaVersion, OllamaErrorCode> {
    match remote {
        Ok(version) => {
            if write_to_path(path, &version).is_err() {
                log::warn!("[ollama-update-check] stage=cache-write code=storage-unavailable");
            }
            Ok(version)
        }
        Err(remote_error) => match read_from_path(path) {
            Ok(Some(version)) => {
                log::warn!("[ollama-update-check] stage=resolve-latest source=cache");
                Ok(version)
            }
            Ok(None) | Err(()) => Err(remote_error),
        },
    }
}

pub(super) fn read_from_path(path: &Path) -> Result<Option<OllamaVersion>, ()> {
    let bytes = match crate::services::private_store::read_bounded_regular(path, MAX_CACHE_BYTES) {
        Ok(crate::services::private_store::BoundedFile::Missing) => return Ok(None),
        Ok(crate::services::private_store::BoundedFile::Content(bytes)) => bytes,
        Err(_) => return Err(()),
    };
    let cached = match serde_json::from_slice::<CachedRelease>(&bytes) {
        Ok(cached) if cached.schema_version == CACHE_SCHEMA_VERSION => cached,
        Ok(_) | Err(_) => {
            log::warn!("[ollama-update-check] stage=cache-read code=invalid-cache");
            return Ok(None);
        }
    };
    Ok(Some(cached.version))
}

fn write_to_path(path: &Path, version: &OllamaVersion) -> Result<(), ()> {
    let bytes = serde_json::to_vec(&CachedRelease {
        schema_version: CACHE_SCHEMA_VERSION,
        version: version.clone(),
    })
    .map_err(|_| ())?;
    crate::services::private_store::atomic_write(path, &bytes).map_err(|_| ())
}
