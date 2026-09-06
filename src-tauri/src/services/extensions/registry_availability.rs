use super::super::error_codes;
use super::{CatalogSnapshot, DynamicIndex, INDEX};

impl DynamicIndex {
    fn availability(&self) -> Result<(), &'static str> {
        if self.available {
            Ok(())
        } else {
            Err(self
                .unavailable_reason
                .unwrap_or(error_codes::REGISTRY_UNAVAILABLE))
        }
    }

    fn refuse(&mut self, reason: &'static str) {
        // Retire every capability together; a failed reload must not leave stale tools usable.
        *self = Self {
            unavailable_reason: Some(reason),
            ..Self::default()
        };
    }
}

pub(crate) fn registry_availability() -> Result<(), &'static str> {
    INDEX
        .read()
        .map_err(|_| error_codes::REGISTRY_UNAVAILABLE)?
        .availability()
}

pub(crate) fn registry_catalog() -> Result<CatalogSnapshot, &'static str> {
    let index = INDEX
        .read()
        .map_err(|_| error_codes::REGISTRY_UNAVAILABLE)?;
    index.availability()?;
    Ok(index.catalog.clone())
}

pub(crate) fn mark_unavailable(error: &str) -> Result<(), String> {
    let reason = match error {
        error_codes::REGISTRY_VERSION_UNSUPPORTED => error_codes::REGISTRY_VERSION_UNSUPPORTED,
        error_codes::REGISTRY_MIGRATION_FAILED => error_codes::REGISTRY_MIGRATION_FAILED,
        _ => error_codes::REGISTRY_UNAVAILABLE,
    };
    INDEX
        .write()
        .map_err(|_| error_codes::REGISTRY_UNAVAILABLE.to_string())?
        .refuse(reason);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn failure_retires_capabilities_and_valid_empty_catalog_recovers() {
        let mut index = DynamicIndex::default();
        assert_eq!(index.availability(), Err(error_codes::REGISTRY_UNAVAILABLE));
        index.available = true;
        index.names.insert("extension.tool".to_string());
        index.replacements.insert("read_file".to_string());
        index.refuse(error_codes::REGISTRY_VERSION_UNSUPPORTED);
        assert_eq!(
            index.availability(),
            Err(error_codes::REGISTRY_VERSION_UNSUPPORTED)
        );
        assert!(index.names.is_empty());
        assert!(index.replacements.is_empty());
        assert!(index.plugins.is_empty());
        assert!(index.tools.is_empty());
        assert!(index.catalog.version.is_empty());
        let catalog =
            super::super::super::discovery_catalog::build(&[], &[], &Default::default()).unwrap();
        index = DynamicIndex {
            available: true,
            catalog,
            ..DynamicIndex::default()
        };
        assert_eq!(index.availability(), Ok(()));
        assert!(!index.catalog.version.is_empty());
    }
}
