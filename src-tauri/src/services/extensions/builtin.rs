use super::types::{
    ExtensionContributions, ExtensionKind, ExtensionManifest, ExtensionRecord, ExtensionStatus,
};
use serde::Deserialize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

const CATALOG: &str =
    include_str!("../../../resources/extension-host/builtin-plugins/catalog.json");
const SOURCE_LABEL: &str = "Beaver";

#[derive(Deserialize)]
struct BuiltinCatalog {
    plugins: Vec<BuiltinDefinition>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct BuiltinDefinition {
    manifest: ExtensionManifest,
    enabled: bool,
    show_in_chat: bool,
}

pub fn records() -> Result<Vec<ExtensionRecord>, String> {
    let catalog: BuiltinCatalog = serde_json::from_str(CATALOG)
        .map_err(|_| "Catalogue de plugins Beaver invalide.".to_string())?;
    let records = catalog
        .plugins
        .into_iter()
        .map(|definition| ExtensionRecord {
            manifest: definition.manifest,
            kind: ExtensionKind::Builtin,
            source: SOURCE_LABEL.to_string(),
            enabled: definition.enabled,
            trusted: true,
            show_in_chat: definition.show_in_chat,
            status: ExtensionStatus::Inactive,
            last_error: None,
            last_activated_at: None,
            contributions: ExtensionContributions::default(),
        })
        .collect::<Vec<_>>();
    super::validation::records(&records)?;
    Ok(records)
}

pub fn merge(mut stored: Vec<ExtensionRecord>) -> Result<Vec<ExtensionRecord>, String> {
    let saved = stored
        .iter()
        .filter(|record| record.kind == ExtensionKind::Builtin)
        .map(|record| (record.manifest.id.clone(), record))
        .collect::<HashMap<_, _>>();
    let mut merged = records()?;
    for record in &mut merged {
        if let Some(previous) = saved.get(&record.manifest.id) {
            record.enabled = previous.enabled;
            record.show_in_chat = previous.show_in_chat;
            record.last_activated_at = previous.last_activated_at.clone();
        }
    }
    stored.retain(|item| item.kind != ExtensionKind::Builtin);
    merged.extend(stored);
    Ok(merged)
}

pub fn resolve_entry(host_directory: &Path, record: &ExtensionRecord) -> Result<PathBuf, String> {
    if record.kind != ExtensionKind::Builtin {
        return Err("Plugin Beaver invalide.".to_string());
    }
    let root = host_directory
        .canonicalize()
        .map_err(|_| "Hôte d'extensions indisponible.".to_string())?;
    let plugin_root = root
        .join("builtin-plugins")
        .canonicalize()
        .map_err(|_| "Catalogue de plugins Beaver indisponible.".to_string())?;
    let main = record
        .manifest
        .main
        .as_deref()
        .ok_or_else(|| "Point d'entrée de plugin manquant.".to_string())?;
    let entry = root
        .join(main)
        .canonicalize()
        .map_err(|_| "Point d'entrée de plugin indisponible.".to_string())?;
    if !entry.starts_with(&plugin_root) || !entry.is_file() {
        return Err("Point d'entrée de plugin invalide.".to_string());
    }
    Ok(entry)
}
