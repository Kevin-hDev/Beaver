#![allow(dead_code)]

use super::constants::MAX_DURABLE_DOCUMENT_BYTES;
use super::durable_fs::{OllamaDurableFs, OllamaFsErrorKind};
use super::error::OllamaErrorCode;
use super::fingerprint::BundleFingerprint;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

const RECEIPT_SCHEMA_VERSION: u8 = 1;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BundlePlatform {
    Macos,
    Linux,
    Windows,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct BundleReceipt {
    pub schema_version: u8,
    pub fingerprint: BundleFingerprint,
    pub platform: BundlePlatform,
}

impl BundleReceipt {
    pub fn new(fingerprint: BundleFingerprint) -> Self {
        Self {
            schema_version: RECEIPT_SCHEMA_VERSION,
            fingerprint,
            platform: current_platform(),
        }
    }

    pub fn parse_bounded(bytes: &[u8]) -> Result<Self, OllamaErrorCode> {
        if bytes.len() > MAX_DURABLE_DOCUMENT_BYTES {
            return Err(OllamaErrorCode::OllamaBundleInvalid);
        }
        let wire: BundleReceiptWire =
            serde_json::from_slice(bytes).map_err(|_| OllamaErrorCode::OllamaBundleInvalid)?;
        if wire.schema_version != RECEIPT_SCHEMA_VERSION {
            return Err(OllamaErrorCode::OllamaBundleInvalid);
        }
        Ok(Self {
            schema_version: wire.schema_version,
            fingerprint: wire.fingerprint,
            platform: wire.platform,
        })
    }

    pub fn validate(&self) -> Result<(), OllamaErrorCode> {
        (self.schema_version == RECEIPT_SCHEMA_VERSION)
            .then_some(())
            .ok_or(OllamaErrorCode::OllamaBundleInvalid)
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct BundleReceiptWire {
    schema_version: u8,
    fingerprint: BundleFingerprint,
    platform: BundlePlatform,
}

pub fn write_version<F: OllamaDurableFs>(
    fs: &F,
    root: &Path,
    version: &str,
) -> Result<(), OllamaErrorCode> {
    if version.is_empty() || version.len() > 64 || version.contains(['/', '\\', '\0']) {
        return Err(OllamaErrorCode::OllamaBundleInvalid);
    }
    let path = root.join("VERSION");
    let tmp = root.join("VERSION.tmp");
    fs.write_new_atomic(&tmp, &path, version.as_bytes())
        .map_err(|_| OllamaErrorCode::OllamaStorageUnavailable)
}

pub fn write_receipt<F: OllamaDurableFs>(
    fs: &F,
    root: &Path,
    receipt: &BundleReceipt,
) -> Result<(), OllamaErrorCode> {
    let (path, tmp) = receipt_paths(root);
    write_receipt_at(fs, &path, &tmp, receipt)
}

pub fn write_receipt_at<F: OllamaDurableFs>(
    fs: &F,
    path: &Path,
    tmp: &Path,
    receipt: &BundleReceipt,
) -> Result<(), OllamaErrorCode> {
    receipt.validate()?;
    let bytes = serde_json::to_vec(receipt).map_err(|_| OllamaErrorCode::OllamaInternal)?;
    if bytes.len() > MAX_DURABLE_DOCUMENT_BYTES {
        return Err(OllamaErrorCode::OllamaBundleInvalid);
    }
    fs.write_new_atomic(tmp, path, &bytes)
        .map_err(|_| OllamaErrorCode::OllamaStorageUnavailable)
}

pub fn read_receipt<F: OllamaDurableFs>(
    fs: &F,
    path: &Path,
) -> Result<Option<BundleReceipt>, OllamaErrorCode> {
    match fs.read_bounded(path, MAX_DURABLE_DOCUMENT_BYTES) {
        Ok(bytes) => BundleReceipt::parse_bounded(&bytes).map(Some),
        Err(error) if error.kind() == OllamaFsErrorKind::NotFound => Ok(None),
        Err(_) => Err(OllamaErrorCode::OllamaStorageUnavailable),
    }
}

pub fn receipt_paths(root: &Path) -> (PathBuf, PathBuf) {
    (
        root.join("ollama-bundle-receipt.json"),
        root.join("ollama-bundle-receipt.tmp"),
    )
}

fn current_platform() -> BundlePlatform {
    #[cfg(target_os = "macos")]
    {
        BundlePlatform::Macos
    }
    #[cfg(target_os = "windows")]
    {
        BundlePlatform::Windows
    }
    #[cfg(target_os = "linux")]
    {
        BundlePlatform::Linux
    }
}
