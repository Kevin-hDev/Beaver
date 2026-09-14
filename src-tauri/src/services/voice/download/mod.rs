mod catalog;
mod catalog_types;
mod catalog_validation;

pub use catalog_types::*;

pub fn load_catalog(
    resource_dir: &std::path::Path,
) -> Result<VoiceCatalog, crate::services::voice::errors::VoiceError> {
    catalog::load_catalog(resource_dir)
}

#[cfg(test)]
mod catalog_tests;
