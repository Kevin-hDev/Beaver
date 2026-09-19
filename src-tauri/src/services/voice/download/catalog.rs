use std::{fs, path::Path};

use super::{catalog_validation::validate_catalog, VoiceCatalog};
use crate::services::voice::{errors::VoiceError, limits};

pub fn load_catalog(resource_dir: &Path) -> Result<VoiceCatalog, VoiceError> {
    let path = [
        resource_dir.join("resources/voice-catalog.json"),
        resource_dir.join("voice-catalog.json"),
    ]
    .into_iter()
    .find(|candidate| candidate.is_file())
    .ok_or_else(|| {
        ::log::warn!("[voice] step=catalog-resource-missing");
        VoiceError::configuration_unavailable()
    })?;

    let metadata = fs::metadata(&path).map_err(|_| VoiceError::configuration_unavailable())?;
    if metadata.len() == 0 || metadata.len() > limits::MAX_CATALOG_BYTES {
        return Err(VoiceError::configuration_unavailable());
    }
    let bytes = fs::read(path).map_err(|_| VoiceError::configuration_unavailable())?;
    let catalog: VoiceCatalog =
        serde_json::from_slice(&bytes).map_err(|_| VoiceError::configuration_unavailable())?;
    validate_catalog(&catalog)?;
    Ok(catalog)
}
