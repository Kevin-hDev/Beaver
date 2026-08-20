use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;
use subtle::ConstantTimeEq;

use super::runtime_environment_fs::{present_dir, Layout};
use super::runtime_error::RuntimeError;
use super::runtime_manifest::RuntimeManifest;

const RECEIPT_NAME: &str = ".runtime.json";
const RECEIPT_TMP_NAME: &str = ".runtime.json.next";
const MAX_RECEIPT_BYTES: u64 = 512;

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Receipt {
    schema_version: u8,
    python_major: u8,
    python_minor: u8,
    requirements_sha256: String,
    source_sha256: String,
}

pub(super) fn reusable(
    layout: &Layout,
    manifest: &RuntimeManifest,
    source_hash: &str,
) -> Result<bool, RuntimeError> {
    if !present_dir(&layout.current)? || !RuntimeManifest::valid_sha256(source_hash) {
        return Ok(false);
    }
    let path = layout.current.join(RECEIPT_NAME);
    let Ok(metadata) = fs::symlink_metadata(&path) else {
        return Ok(false);
    };
    if !metadata.file_type().is_file()
        || !single_link(&metadata)
        || metadata.len() == 0
        || metadata.len() > MAX_RECEIPT_BYTES
    {
        return Ok(false);
    }
    let Ok(bytes) = fs::read(path) else {
        return Ok(false);
    };
    let Ok(receipt) = serde_json::from_slice::<Receipt>(&bytes) else {
        return Ok(false);
    };
    Ok(receipt.schema_version == 1
        && receipt.python_major == manifest.python_major
        && receipt.python_minor == manifest.python_minor
        && manifest.matches_stamp(&receipt.requirements_sha256)
        && RuntimeManifest::valid_sha256(&receipt.source_sha256)
        && receipt
            .source_sha256
            .as_bytes()
            .ct_eq(source_hash.as_bytes())
            .into())
}

pub(super) fn write_receipt(
    layout: &Layout,
    manifest: &RuntimeManifest,
    source_hash: &str,
) -> Result<(), RuntimeError> {
    if !RuntimeManifest::valid_sha256(source_hash) || !present_dir(&layout.staged)? {
        return Err(RuntimeError::EnvironmentUnavailable);
    }
    let receipt = Receipt {
        schema_version: 1,
        python_major: manifest.python_major,
        python_minor: manifest.python_minor,
        requirements_sha256: manifest.requirements_sha256().to_string(),
        source_sha256: source_hash.to_string(),
    };
    let body = serde_json::to_vec(&receipt).map_err(|_| RuntimeError::EnvironmentUnavailable)?;
    if body.len() as u64 > MAX_RECEIPT_BYTES {
        return Err(RuntimeError::EnvironmentUnavailable);
    }
    let final_path = layout.staged.join(RECEIPT_NAME);
    let temp_path = layout.staged.join(RECEIPT_TMP_NAME);
    if fs::symlink_metadata(&final_path).is_ok() || fs::symlink_metadata(&temp_path).is_ok() {
        return Err(RuntimeError::EnvironmentUnavailable);
    }
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temp_path)
        .map_err(|_| RuntimeError::EnvironmentUnavailable)?;
    file.write_all(&body)
        .map_err(|_| RuntimeError::EnvironmentUnavailable)?;
    file.sync_all()
        .map_err(|_| RuntimeError::EnvironmentUnavailable)?;
    fs::rename(temp_path, final_path).map_err(|_| RuntimeError::EnvironmentUnavailable)
}

pub(super) fn source_hash(source: &Path) -> Result<String, RuntimeError> {
    let required = ["setup.py", "requirements.txt", "LICENSE", "searx/webapp.py"];
    if !required.iter().all(|file| source.join(file).is_file()) {
        return Err(RuntimeError::WheelhouseUnavailable);
    }
    let mut hasher = Sha256::new();
    for file in ["setup.py", "requirements.txt", "searx/version.py"] {
        hasher
            .update(fs::read(source.join(file)).map_err(|_| RuntimeError::WheelhouseUnavailable)?);
    }
    Ok(hex::encode(hasher.finalize()))
}

#[cfg(unix)]
fn single_link(metadata: &fs::Metadata) -> bool {
    use std::os::unix::fs::MetadataExt;

    metadata.nlink() == 1
}

#[cfg(not(unix))]
fn single_link(_: &fs::Metadata) -> bool {
    true
}
