use super::types::{ExtensionKind, ExtensionRecord, MAX_EXTENSIONS, MAX_USER_EXTENSIONS};
use super::OperationFailure;

pub fn add(record: ExtensionRecord) -> Result<(), OperationFailure> {
    super::validation::records(std::slice::from_ref(&record))
        .map_err(|_| OperationFailure::ManifestInvalid)?;
    super::registry::mutate(|records| {
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
