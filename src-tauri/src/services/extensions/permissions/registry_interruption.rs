use super::types::{ExtensionContributions, ExtensionRecord, ExtensionStatus};

pub(super) fn mark_interrupted(extension_id: &str) -> Result<(), String> {
    super::validation::identifier(extension_id)?;
    super::registry::mutate(|records| {
        mark_interrupted_records(records, extension_id);
        Ok::<(), String>(())
    })
}

pub(super) fn mark_interrupted_records(records: &mut [ExtensionRecord], extension_id: &str) {
    if let Some(record) = records
        .iter_mut()
        .find(|record| record.manifest.id == extension_id)
    {
        record.status = ExtensionStatus::Error;
        record.last_error = Some(super::error_codes::LOAD_INTERRUPTED.to_string());
        record.contributions = ExtensionContributions::default();
    }
}
