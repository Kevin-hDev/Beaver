#![allow(dead_code)]

use super::constants::MAX_DURABLE_DOCUMENT_BYTES;
use super::fingerprint::BundleFingerprint;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DocumentError {
    Oversized,
    InvalidJson,
    UnsupportedSchema,
}

impl fmt::Display for DocumentError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::Oversized => "durable document is oversized",
            Self::InvalidJson => "durable document is invalid",
            Self::UnsupportedSchema => "durable document schema is unsupported",
        };
        formatter.write_str(message)
    }
}

impl std::error::Error for DocumentError {}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "phase", deny_unknown_fields)]
pub enum OllamaJournalState {
    Prepared {
        target: BundleFingerprint,
        previous: BundleFingerprint,
    },
    PendingValidation {
        target: BundleFingerprint,
        previous: BundleFingerprint,
    },
    CleanupPending {
        target: BundleFingerprint,
        previous: BundleFingerprint,
    },
    RollbackPending {
        previous: BundleFingerprint,
        rejected_target: Option<BundleFingerprint>,
    },
    RollbackCleanupPending {
        previous: BundleFingerprint,
        rejected_target: Option<BundleFingerprint>,
    },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct OllamaTransactionJournal {
    pub schema_version: u8,
    #[serde(flatten)]
    pub state: OllamaJournalState,
}

impl<'de> Deserialize<'de> for OllamaTransactionJournal {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = serde_json::Value::deserialize(deserializer)?;
        parse_journal_value(value).map_err(serde::de::Error::custom)
    }
}

impl OllamaTransactionJournal {
    pub fn new(state: OllamaJournalState) -> Self {
        Self {
            schema_version: 1,
            state,
        }
    }

    pub fn parse_bounded(bytes: &[u8]) -> Result<Self, DocumentError> {
        if bytes.len() > MAX_DURABLE_DOCUMENT_BYTES {
            return Err(DocumentError::Oversized);
        }
        let value = serde_json::from_slice(bytes).map_err(|_| DocumentError::InvalidJson)?;
        parse_journal_value(value)
    }

    pub fn validate(&self) -> Result<(), DocumentError> {
        (self.schema_version == 1)
            .then_some(())
            .ok_or(DocumentError::UnsupportedSchema)
    }
}

fn parse_journal_value(
    value: serde_json::Value,
) -> Result<OllamaTransactionJournal, DocumentError> {
    let object = value.as_object().ok_or(DocumentError::InvalidJson)?;
    let schema_version = object
        .get("schema_version")
        .and_then(serde_json::Value::as_u64)
        .and_then(|value| u8::try_from(value).ok())
        .ok_or(DocumentError::InvalidJson)?;
    let phase = object
        .get("phase")
        .and_then(serde_json::Value::as_str)
        .ok_or(DocumentError::InvalidJson)?;
    let allowed = match phase {
        "Prepared" | "PendingValidation" | "CleanupPending" => {
            &["schema_version", "phase", "target", "previous"][..]
        }
        "RollbackPending" | "RollbackCleanupPending" => {
            &["schema_version", "phase", "previous", "rejected_target"][..]
        }
        _ => return Err(DocumentError::InvalidJson),
    };
    if object.keys().any(|key| !allowed.contains(&key.as_str())) {
        return Err(DocumentError::InvalidJson);
    }
    if matches!(phase, "RollbackPending" | "RollbackCleanupPending")
        && !object.contains_key("rejected_target")
    {
        return Err(DocumentError::InvalidJson);
    }
    let mut state_object = object.clone();
    state_object.remove("schema_version");
    let state = serde_json::from_value(serde_json::Value::Object(state_object))
        .map_err(|_| DocumentError::InvalidJson)?;
    let document = OllamaTransactionJournal {
        schema_version,
        state,
    };
    document.validate()?;
    Ok(document)
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OllamaMigrationMarker {
    pub schema_version: u8,
    pub legacy_layout_migrated: bool,
}

impl OllamaMigrationMarker {
    pub fn new() -> Self {
        Self {
            schema_version: 1,
            legacy_layout_migrated: true,
        }
    }

    pub fn parse_bounded(bytes: &[u8]) -> Result<Self, DocumentError> {
        if bytes.len() > MAX_DURABLE_DOCUMENT_BYTES {
            return Err(DocumentError::Oversized);
        }
        let marker: Self = serde_json::from_slice(bytes).map_err(|_| DocumentError::InvalidJson)?;
        marker.validate()?;
        Ok(marker)
    }

    pub fn validate(&self) -> Result<(), DocumentError> {
        (self.schema_version == 1 && self.legacy_layout_migrated)
            .then_some(())
            .ok_or(DocumentError::UnsupportedSchema)
    }
}

impl Default for OllamaMigrationMarker {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum OllamaMigrationMarkerClassification {
    Absent,
    Valid(OllamaMigrationMarker),
    Invalid,
}

pub fn classify_migration_marker(bytes: Option<&[u8]>) -> OllamaMigrationMarkerClassification {
    match bytes {
        None => OllamaMigrationMarkerClassification::Absent,
        Some(bytes) => OllamaMigrationMarker::parse_bounded(bytes)
            .map(OllamaMigrationMarkerClassification::Valid)
            .unwrap_or(OllamaMigrationMarkerClassification::Invalid),
    }
}
