use crate::services::extensions::{self, ExtensionKind};

pub(super) fn open(extension_id: &str) -> Result<(), String> {
    extensions::validate_identifier(extension_id)?;
    let record = extensions::list()?
        .into_iter()
        .find(|record| record.manifest.id == extension_id)
        .ok_or_else(|| extensions::error_codes::NOT_FOUND.to_string())?;
    if record.kind != ExtensionKind::Local {
        return Err(extensions::error_codes::OPERATION_FAILED.to_string());
    }
    let source = std::path::PathBuf::from(record.source)
        .canonicalize()
        .map_err(|_| extensions::error_codes::NOT_FOUND.to_string())?;
    open::that_detached(source).map_err(|_| extensions::error_codes::OPERATION_FAILED.to_string())
}
