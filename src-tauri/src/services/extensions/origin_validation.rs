use super::types::{ExtensionKind, ExtensionOriginKind, ExtensionRecord};

pub fn record(record: &ExtensionRecord) -> Result<(), String> {
    let Some(origin) = record.origin.as_ref() else {
        return Ok(());
    };
    if record.kind != ExtensionKind::Local {
        return Err("Provenance d'extension invalide.".to_string());
    }
    match origin.kind {
        ExtensionOriginKind::Local => {
            super::validation::source_input(&origin.locator)?;
            if origin.locator != record.source || origin.revision.is_some() {
                return Err("Provenance locale invalide.".to_string());
            }
        }
        ExtensionOriginKind::Git => {
            super::source_validation::git(&origin.locator)?;
            if !origin.revision.as_deref().is_some_and(valid_revision) {
                return Err("Révision Git invalide.".to_string());
            }
        }
        ExtensionOriginKind::Npm => {
            super::source_validation::npm(&origin.locator)?;
            if origin.revision.is_some() {
                return Err("Provenance npm invalide.".to_string());
            }
        }
    }
    Ok(())
}

fn valid_revision(value: &str) -> bool {
    matches!(value.len(), 40 | 64)
        && value
            .chars()
            .all(|character| character.is_ascii_hexdigit() && !character.is_ascii_uppercase())
}
