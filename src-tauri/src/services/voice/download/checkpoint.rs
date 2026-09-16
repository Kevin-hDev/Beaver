use std::{fs, path::Path, time::Duration};

use serde::{Deserialize, Serialize};

use crate::services::private_store::{
    atomic_write_with_durability, read_bounded_regular, BoundedFile, PublicationDurability,
};

const CHECKPOINT_VERSION: u8 = 1;
const MAX_CHECKPOINT_BYTES: u64 = 4 * 1024;
const MAX_VALIDATOR_CHARS: usize = 512;
const CHECKPOINT_BYTES: u64 = 4 * 1024 * 1024;
const CHECKPOINT_INTERVAL: Duration = Duration::from_secs(5);

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Checkpoint {
    pub version: u8,
    pub requested_model_id: String,
    pub entry_id: String,
    pub revision: String,
    pub expected_bytes: u64,
    pub expected_sha256: String,
    pub durable_bytes: u64,
    pub validator: Option<HttpValidator>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", tag = "kind", content = "value")]
pub enum HttpValidator {
    Etag(String),
    LastModified(String),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PartialDecision {
    Resume { durable_bytes: u64 },
    Truncate { durable_bytes: u64 },
    Discard,
}

impl Checkpoint {
    pub fn new(
        requested_model_id: String,
        entry_id: String,
        revision: String,
        expected_bytes: u64,
        expected_sha256: String,
    ) -> Option<Self> {
        let checkpoint = Self {
            version: CHECKPOINT_VERSION,
            requested_model_id,
            entry_id,
            revision,
            expected_bytes,
            expected_sha256,
            durable_bytes: 0,
            validator: None,
        };
        checkpoint.is_valid().then_some(checkpoint)
    }

    pub fn is_valid(&self) -> bool {
        self.version == CHECKPOINT_VERSION
            && valid_id(&self.requested_model_id)
            && valid_id(&self.entry_id)
            && valid_revision(&self.revision)
            && self.expected_bytes > 0
            && self.durable_bytes <= self.expected_bytes
            && valid_sha(&self.expected_sha256)
            && self.validator.as_ref().is_none_or(valid_validator)
    }

    pub fn matches_entry(&self, entry: &super::VoiceCatalogEntry) -> bool {
        self.entry_id == entry.id
            && self.revision == entry.revision
            && self.expected_bytes == entry.archive.bytes
            && self.expected_sha256 == entry.archive.sha256
    }
}

pub fn inspect_partial(checkpoint: &Checkpoint, actual_len: Option<u64>) -> PartialDecision {
    if !checkpoint.is_valid() {
        return PartialDecision::Discard;
    }
    match actual_len {
        Some(actual) if actual == checkpoint.durable_bytes => PartialDecision::Resume {
            durable_bytes: actual,
        },
        Some(actual) if actual > checkpoint.durable_bytes => PartialDecision::Truncate {
            durable_bytes: checkpoint.durable_bytes,
        },
        _ => PartialDecision::Discard,
    }
}

pub(super) fn load_checkpoint(path: &Path) -> Result<Option<Checkpoint>, String> {
    match read_bounded_regular(path, MAX_CHECKPOINT_BYTES)? {
        BoundedFile::Missing => Ok(None),
        BoundedFile::Content(bytes) => {
            let checkpoint: Checkpoint = serde_json::from_slice(&bytes)
                .map_err(|_| "model-download-checkpoint-invalid".to_string())?;
            checkpoint
                .is_valid()
                .then_some(Some(checkpoint))
                .ok_or_else(|| "model-download-checkpoint-invalid".to_string())
        }
    }
}

pub(crate) fn save_checkpoint(path: &Path, checkpoint: &Checkpoint) -> Result<(), String> {
    if !checkpoint.is_valid() {
        return Err("model-download-checkpoint-invalid".into());
    }
    let bytes = serde_json::to_vec(checkpoint)
        .map_err(|_| "model-download-checkpoint-invalid".to_string())?;
    match atomic_write_with_durability(path, &bytes)? {
        PublicationDurability::Durable => Ok(()),
        PublicationDurability::PublishedDurabilityUnconfirmed => {
            Err("model-download-durability-unconfirmed".into())
        }
    }
}

pub(crate) fn discover_checkpoints(data_dir: &Path) -> Vec<Checkpoint> {
    let directory = super::models_root(data_dir).join(super::PARTIALS_DIR);
    let Ok(entries) = fs::read_dir(directory) else {
        return Vec::new();
    };
    entries
        .take(crate::services::model_downloads_types::MAX_PENDING_DOWNLOADS * 4)
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_ok_and(|kind| kind.is_file()))
        .filter_map(|entry| {
            let checkpoint = load_checkpoint(&entry.path()).ok().flatten()?;
            (entry.file_name().to_string_lossy()
                == format!("{}.checkpoint.json", checkpoint.entry_id))
            .then_some(checkpoint)
        })
        .take(crate::services::model_downloads_types::MAX_PENDING_DOWNLOADS)
        .collect()
}

pub(super) fn should_checkpoint(received: u64, confirmed: u64, elapsed: Duration) -> bool {
    received > confirmed
        && (received - confirmed >= CHECKPOINT_BYTES || elapsed >= CHECKPOINT_INTERVAL)
}

fn valid_validator(validator: &HttpValidator) -> bool {
    let value = match validator {
        HttpValidator::Etag(value) | HttpValidator::LastModified(value) => value,
    };
    !value.is_empty()
        && value.chars().count() <= MAX_VALIDATOR_CHARS
        && !value.chars().any(char::is_control)
}

fn valid_id(value: &str) -> bool {
    !value.is_empty()
        && value.chars().count() <= super::super::limits::MAX_CATALOG_ID_CHARS
        && value.chars().all(|character| {
            character.is_ascii_lowercase() || character.is_ascii_digit() || character == '-'
        })
}

fn valid_revision(value: &str) -> bool {
    if value.len() == 40 {
        return is_lower_hex(value);
    }
    value
        .strip_prefix("sha256:")
        .is_some_and(|digest| digest.len() == 64 && is_lower_hex(digest))
}

fn valid_sha(value: &str) -> bool {
    value.len() == 64 && is_lower_hex(value)
}

fn is_lower_hex(value: &str) -> bool {
    value
        .bytes()
        .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
}
