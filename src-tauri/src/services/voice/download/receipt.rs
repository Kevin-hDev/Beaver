use std::{
    collections::HashSet,
    path::{Component, Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use crate::services::private_store::{
    atomic_write_with_durability, read_bounded_regular, BoundedFile, PublicationDurability,
};

use super::{
    catalog_validation::revision_directory_name, models_root, VoiceCatalogEntry, VoiceModelFile,
};

const RECEIPT_VERSION: u8 = 1;
const MAX_RECEIPT_BYTES: u64 = 32 * 1024;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InstallationReceipt {
    pub version: u8,
    pub entry_id: String,
    pub revision: String,
    pub installed_bytes: u64,
    pub files: Vec<ReceiptFile>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReceiptFile {
    pub path: String,
    pub bytes: u64,
    pub sha256: String,
}

impl InstallationReceipt {
    pub(super) fn from_entry(entry: &VoiceCatalogEntry) -> Self {
        Self {
            version: RECEIPT_VERSION,
            entry_id: entry.id.clone(),
            revision: entry.revision.clone(),
            installed_bytes: entry.installed_bytes,
            files: entry.files.iter().map(ReceiptFile::from).collect(),
        }
    }

    pub fn install_dir(&self, data_dir: &Path) -> PathBuf {
        models_root(data_dir)
            .join("models")
            .join(&self.entry_id)
            .join(revision_directory_name(&self.revision).unwrap_or("invalid-revision"))
    }
}

impl From<&VoiceModelFile> for ReceiptFile {
    fn from(file: &VoiceModelFile) -> Self {
        Self {
            path: file.path.clone(),
            bytes: file.bytes,
            sha256: file.sha256.clone(),
        }
    }
}

pub(super) fn receipt_path(data_dir: &Path, id: &str) -> PathBuf {
    models_root(data_dir)
        .join("receipts")
        .join(format!("{id}.json"))
}

pub(super) fn save_receipt(data_dir: &Path, receipt: &InstallationReceipt) -> Result<(), String> {
    let bytes = serde_json::to_vec(receipt).map_err(|_| receipt_error())?;
    match atomic_write_with_durability(&receipt_path(data_dir, &receipt.entry_id), &bytes)? {
        PublicationDurability::Durable => Ok(()),
        PublicationDurability::PublishedDurabilityUnconfirmed => Err(receipt_error()),
    }
}

pub(super) fn load_receipt(
    data_dir: &Path,
    id: &str,
) -> Result<Option<InstallationReceipt>, String> {
    match read_bounded_regular(&receipt_path(data_dir, id), MAX_RECEIPT_BYTES)? {
        BoundedFile::Missing => Ok(None),
        BoundedFile::Content(bytes) => {
            let receipt = serde_json::from_slice::<InstallationReceipt>(&bytes)
                .map_err(|_| receipt_error())?;
            receipt_valid(&receipt)
                .then_some(Some(receipt))
                .ok_or_else(receipt_error)
        }
    }
}

pub(super) fn receipt_matches(receipt: &InstallationReceipt, entry: &VoiceCatalogEntry) -> bool {
    receipt == &InstallationReceipt::from_entry(entry)
}

fn receipt_valid(receipt: &InstallationReceipt) -> bool {
    let mut paths = HashSet::with_capacity(receipt.files.len());
    let total = receipt
        .files
        .iter()
        .try_fold(0_u64, |sum, file| sum.checked_add(file.bytes));
    receipt.version == RECEIPT_VERSION
        && !receipt.entry_id.is_empty()
        && revision_directory_name(&receipt.revision).is_some()
        && receipt.entry_id.chars().all(|character| {
            character.is_ascii_lowercase() || character.is_ascii_digit() || character == '-'
        })
        && receipt.files.len() <= crate::services::voice::limits::MAX_CATALOG_FILES_PER_ENTRY
        && !receipt.files.is_empty()
        && total == Some(receipt.installed_bytes)
        && receipt.files.iter().all(|file| {
            !file.path.is_empty()
                && file.bytes > 0
                && file.sha256.len() == 64
                && file
                    .sha256
                    .bytes()
                    .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
                && Path::new(&file.path)
                    .components()
                    .all(|component| matches!(component, Component::Normal(_)))
                && paths.insert(&file.path)
        })
}

fn receipt_error() -> String {
    "model-download-receipt-invalid".into()
}
