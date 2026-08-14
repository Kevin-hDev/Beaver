#![allow(dead_code)]

use super::constants::MAX_DURABLE_DOCUMENT_BYTES;
use super::fingerprint::BundleFingerprint;
use serde::Serialize;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DocumentError {
    Oversized,
    InvalidJson,
    UnsupportedSchema,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
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
pub struct OllamaTransactionJournal {
    pub schema_version: u8,
    #[serde(flatten)]
    pub state: OllamaJournalState,
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
        super::journal_wire::parse_journal(bytes)
    }

    pub fn validate(&self) -> Result<(), DocumentError> {
        (self.schema_version == 1)
            .then_some(())
            .ok_or(DocumentError::UnsupportedSchema)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
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
        let (schema_version, legacy_layout_migrated) = super::journal_wire::parse_marker(bytes)?;
        let marker = Self {
            schema_version,
            legacy_layout_migrated,
        };
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
