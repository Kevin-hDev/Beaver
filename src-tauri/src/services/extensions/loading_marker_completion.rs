use std::collections::HashSet;
use std::path::Path;

use super::{marker_error, JournalRead, MarkerRead, PreservedMarker};
use crate::services::extensions::loading_journal_store as store;
use crate::services::extensions::loading_marker_format::LoadingMarker;

pub(super) fn complete_at(
    marker_path: &Path,
    preserved: PreservedMarker,
    applied_ids: &HashSet<String>,
    resolved_recovery: Option<(&str, u8)>,
) -> Result<(), String> {
    store::transaction(marker_path, false, |read| {
        let mut journal = match read {
            JournalRead::Valid(journal) => journal,
            JournalRead::Missing if applied_ids.is_empty() => return Ok(None),
            JournalRead::Invalid
                if applied_ids.is_empty() && matches!(preserved.state, MarkerRead::Invalid) =>
            {
                return Ok(None)
            }
            JournalRead::Missing | JournalRead::Invalid => return Err(marker_error()),
        };
        let current = journal.host().cloned();
        if let Some((extension_id, attempts)) = resolved_recovery {
            if applied_ids.contains(extension_id) {
                *journal.host_mut() = None;
            } else if !matches!(
                current,
                Some(ref marker)
                    if marker.extension_id == extension_id && marker.attempts == attempts
            ) {
                *journal.host_mut() = Some(LoadingMarker::new_host(extension_id, attempts)?);
            }
            return Ok(Some(journal));
        }
        let Some(current) = current else {
            return Ok(None);
        };
        let preserved_is_same = preserved
            .host
            .as_ref()
            .is_some_and(|marker| marker.extension_id == current.extension_id);
        if applied_ids.contains(&current.extension_id) {
            *journal.host_mut() = if preserved_is_same {
                None
            } else {
                preserved.host
            };
        } else if !preserved_is_same && preserved.host.is_some() {
            *journal.host_mut() = preserved.host;
        }
        Ok(Some(journal))
    })
}
