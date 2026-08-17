#![allow(dead_code)]

use super::super::fingerprint::{BundleFingerprint, OllamaVersion, Sha256Digest};
use super::{DocumentError, OllamaJournalState, OllamaTransactionJournal};
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct JournalWire {
    schema_version: u8,
    phase: JournalPhaseWire,
    #[serde(default)]
    target: WireField<BundleFingerprintWire>,
    #[serde(default)]
    previous: Option<BundleFingerprintWire>,
    #[serde(default)]
    rejected_target: WireField<BundleFingerprintWire>,
}

#[derive(Deserialize)]
enum JournalPhaseWire {
    Prepared,
    PendingValidation,
    CleanupPending,
    RollbackPending,
    RollbackCleanupPending,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct BundleFingerprintWire {
    version: OllamaVersion,
    executable_sha256: Sha256Digest,
}

#[derive(Default)]
enum WireField<T> {
    #[default]
    Missing,
    Null,
    Value(T),
}

impl<'de, T: Deserialize<'de>> Deserialize<'de> for WireField<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(match Option::<T>::deserialize(deserializer)? {
            Some(value) => Self::Value(value),
            None => Self::Null,
        })
    }
}

impl BundleFingerprintWire {
    fn into_public(self) -> BundleFingerprint {
        BundleFingerprint {
            version: self.version,
            executable_sha256: self.executable_sha256,
        }
    }
}

pub(super) fn parse_journal(bytes: &[u8]) -> Result<OllamaTransactionJournal, DocumentError> {
    let wire: JournalWire =
        serde_json::from_slice(bytes).map_err(|_| DocumentError::InvalidJson)?;
    from_journal_wire(wire)
}

pub(super) fn parse_marker(bytes: &[u8]) -> Result<(u8, bool), DocumentError> {
    let wire: MigrationMarkerWire =
        serde_json::from_slice(bytes).map_err(|_| DocumentError::InvalidJson)?;
    Ok((wire.schema_version, wire.legacy_layout_migrated))
}

fn from_journal_wire(wire: JournalWire) -> Result<OllamaTransactionJournal, DocumentError> {
    let JournalWire {
        schema_version,
        phase,
        target,
        previous,
        rejected_target,
    } = wire;
    let previous = previous.ok_or(DocumentError::InvalidJson)?;
    let rollback = matches!(
        &phase,
        JournalPhaseWire::RollbackPending | JournalPhaseWire::RollbackCleanupPending
    );
    if (rollback && !matches!(target, WireField::Missing))
        || (!rollback && !matches!(rejected_target, WireField::Missing))
    {
        return Err(DocumentError::InvalidJson);
    }
    let state = match phase {
        JournalPhaseWire::Prepared => OllamaJournalState::Prepared {
            target: required(target)?.into_public(),
            previous: previous.into_public(),
        },
        JournalPhaseWire::PendingValidation => OllamaJournalState::PendingValidation {
            target: required(target)?.into_public(),
            previous: previous.into_public(),
        },
        JournalPhaseWire::CleanupPending => OllamaJournalState::CleanupPending {
            target: required(target)?.into_public(),
            previous: previous.into_public(),
        },
        JournalPhaseWire::RollbackPending => OllamaJournalState::RollbackPending {
            previous: previous.into_public(),
            rejected_target: optional(rejected_target)?.map(BundleFingerprintWire::into_public),
        },
        JournalPhaseWire::RollbackCleanupPending => OllamaJournalState::RollbackCleanupPending {
            previous: previous.into_public(),
            rejected_target: optional(rejected_target)?.map(BundleFingerprintWire::into_public),
        },
    };
    let document = OllamaTransactionJournal {
        schema_version,
        state,
    };
    document.validate()?;
    Ok(document)
}

fn required<T>(field: WireField<T>) -> Result<T, DocumentError> {
    match field {
        WireField::Value(value) => Ok(value),
        WireField::Missing | WireField::Null => Err(DocumentError::InvalidJson),
    }
}

fn optional<T>(field: WireField<T>) -> Result<Option<T>, DocumentError> {
    match field {
        WireField::Null => Ok(None),
        WireField::Value(value) => Ok(Some(value)),
        WireField::Missing => Err(DocumentError::InvalidJson),
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct MigrationMarkerWire {
    schema_version: u8,
    legacy_layout_migrated: bool,
}
