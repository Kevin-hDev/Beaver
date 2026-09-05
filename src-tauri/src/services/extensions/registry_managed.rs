use super::types::{ExtensionKind, ExtensionRecord, MAX_EXTENSIONS, MAX_USER_EXTENSIONS};
use super::OperationFailure;

pub fn add(record: ExtensionRecord) -> Result<(), OperationFailure> {
    super::validation::records(std::slice::from_ref(&record))
        .map_err(|_| OperationFailure::ManifestInvalid)?;
    super::registry::mutate(|records| {
        ensure_current(&record).map_err(|_| OperationFailure::ManifestInvalid)?;
        let user_extensions = records
            .iter()
            .filter(|item| item.kind != ExtensionKind::Builtin)
            .count();
        if user_extensions >= MAX_USER_EXTENSIONS || records.len() >= MAX_EXTENSIONS {
            return Err(OperationFailure::LimitReached);
        }
        if records
            .iter()
            .any(|item| item.manifest.id == record.manifest.id)
        {
            return Err(OperationFailure::AlreadyInstalled);
        }
        records.push(record);
        Ok(())
    })
}

pub(super) fn ensure_current(record: &ExtensionRecord) -> Result<(), String> {
    let root = std::path::Path::new(&record.source);
    let input = if super::manifest_source::manifest_path(root).is_some() {
        root.to_path_buf()
    } else {
        root.join(
            record
                .manifest
                .main
                .as_deref()
                .ok_or(super::error_codes::MANIFEST_INVALID)?,
        )
    };
    let loaded =
        super::manifest::load_local(input.to_str().ok_or(super::error_codes::MANIFEST_INVALID)?)?;
    if loaded.record.manifest.id != record.manifest.id {
        return Err(super::error_codes::UPDATE_IDENTITY_CHANGED.into());
    }
    // Keep the approved partial, point-in-time fingerprint check at publication.
    if super::fingerprint::is_current(record)? {
        Ok(())
    } else {
        Err(super::error_codes::FINGERPRINT_CHANGED.into())
    }
}
