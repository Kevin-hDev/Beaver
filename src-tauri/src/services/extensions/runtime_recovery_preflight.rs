use super::types::{ExtensionKind, ExtensionRecord};

#[derive(Clone, Debug)]
pub(super) enum RecoveryPreflight {
    Normal,
    Interrupted(String),
    Invalid,
    Retry(String, u8),
}

impl RecoveryPreflight {
    pub(super) fn resolve_for(self, records: &[ExtensionRecord]) -> Result<Self, String> {
        let target = |id: &str| records.iter().find(|record| record.manifest.id == id);
        match self {
            Self::Interrupted(id) if target(&id).is_none() => Ok(Self::Invalid),
            Self::Retry(id, _)
                if target(&id).is_none_or(|record| !record.enabled || !record.trusted) =>
            {
                Err(super::error_codes::RECOVERY_MARKER_INVALID.to_string())
            }
            recovery => Ok(recovery),
        }
    }

    pub(super) fn validate_retry_marker(
        &self,
        marker: &super::loading_marker::MarkerRead,
    ) -> Result<(), String> {
        let Self::Retry(id, attempts) = self else {
            return Ok(());
        };
        match marker {
            super::loading_marker::MarkerRead::Valid(marker)
                if marker.extension_id == *id
                    && marker.can_retry()
                    && marker.attempts.checked_add(1) == Some(*attempts) =>
            {
                Ok(())
            }
            _ => Err(super::error_codes::RECOVERY_MARKER_INVALID.to_string()),
        }
    }

    pub(super) fn attempts_for(&self, extension_id: &str) -> u8 {
        match self {
            Self::Retry(id, attempts) if id == extension_id => *attempts,
            _ => 1,
        }
    }

    pub(super) fn retry_details(&self) -> Option<(&str, u8)> {
        match self {
            Self::Retry(id, attempts) => Some((id, *attempts)),
            _ => None,
        }
    }
}

pub(super) fn filter_for_recovery(
    records: Vec<ExtensionRecord>,
    recovery: &RecoveryPreflight,
) -> Vec<ExtensionRecord> {
    records
        .into_iter()
        .filter(|record| match recovery {
            RecoveryPreflight::Normal => true,
            RecoveryPreflight::Interrupted(id) => record.manifest.id != *id,
            RecoveryPreflight::Invalid => record.kind == ExtensionKind::Builtin,
            RecoveryPreflight::Retry(_, _) => true,
        })
        .collect()
}
