use std::collections::HashSet;
use std::path::Path;

use super::{
    discard_at, marker_error, read_at, write_at, write_bytes_at, LoadingMarker, MarkerRead,
    PreservedMarker,
};

pub(super) fn complete_at(
    marker_path: &Path,
    preserved: PreservedMarker,
    applied_ids: &HashSet<String>,
    resolved_recovery: Option<(&str, u8)>,
) -> Result<(), String> {
    let current = match read_at(marker_path) {
        MarkerRead::Valid(current) => current,
        MarkerRead::Missing if applied_ids.is_empty() => return Ok(()),
        MarkerRead::Invalid
            if applied_ids.is_empty() && matches!(preserved.state, MarkerRead::Invalid) =>
        {
            return Ok(())
        }
        MarkerRead::Missing | MarkerRead::Invalid => return Err(marker_error()),
    };
    if let Some((extension_id, attempts)) = resolved_recovery {
        if applied_ids.contains(extension_id) {
            return discard_at(marker_path);
        }
        if current.extension_id == extension_id && current.attempts == attempts {
            return Ok(());
        }
        let retry = LoadingMarker::new(extension_id, attempts)?;
        return write_at(marker_path, &retry, false);
    }
    if !applied_ids.contains(&current.extension_id) {
        if matches!(
            preserved.state,
            MarkerRead::Valid(ref marker) if marker.extension_id != current.extension_id
        ) {
            if let Some(bytes) = preserved.bytes {
                return write_bytes_at(marker_path, &bytes, false);
            }
        }
        return Ok(());
    }
    match preserved.bytes {
        Some(bytes)
            if !matches!(
                preserved.state,
                MarkerRead::Valid(ref marker) if marker.extension_id == current.extension_id
            ) =>
        {
            write_bytes_at(marker_path, &bytes, false)
        }
        _ => discard_at(marker_path),
    }
}
