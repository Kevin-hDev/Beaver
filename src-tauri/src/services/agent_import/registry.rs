use super::limits::{
    MAX_ITEM_ID_BYTES, MAX_PATH_BYTES, MAX_RULES_PER_SOURCE, MAX_SKILLS_PER_SOURCE, MAX_SOURCES,
};
use super::models::SourceSelection;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

const REGISTRY_VERSION: u8 = 1;
const MAX_REGISTRY_BYTES: u64 = 1024 * 1024;
const SOURCE_IDS: &[&str] = &[
    "claude", "codex", "agents", "hermes", "qwen", "zcode", "openclaw", "opencode", "kimi",
];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportedDocument {
    pub name: String,
    pub source_id: String,
    pub source_path: PathBuf,
    pub source_hash: String,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentImportRegistry {
    pub version: u8,
    #[serde(default)]
    pub sources: Vec<SourceSelection>,
    #[serde(default)]
    pub documents: Vec<ImportedDocument>,
}

impl Default for AgentImportRegistry {
    fn default() -> Self {
        Self {
            version: REGISTRY_VERSION,
            sources: Vec::new(),
            documents: Vec::new(),
        }
    }
}

pub fn registry_path() -> PathBuf {
    crate::services::paths::data_dir().join("external-agent-sources.json")
}

pub fn read() -> AgentImportRegistry {
    read_from(&registry_path())
}

pub fn read_from(path: &Path) -> AgentImportRegistry {
    let Ok(metadata) = std::fs::metadata(path) else {
        return AgentImportRegistry::default();
    };
    if metadata.len() > MAX_REGISTRY_BYTES {
        return AgentImportRegistry::default();
    }
    let Ok(bytes) = std::fs::read(path) else {
        return AgentImportRegistry::default();
    };
    let Ok(registry) = serde_json::from_slice::<AgentImportRegistry>(&bytes) else {
        return AgentImportRegistry::default();
    };
    if validate_registry(&registry).is_err() {
        return AgentImportRegistry::default();
    }
    registry
}

pub fn write_to(path: &Path, registry: &AgentImportRegistry) -> Result<(), String> {
    validate_registry(registry)?;
    let bytes = serde_json::to_vec_pretty(registry).map_err(|_| "Configuration invalide")?;
    crate::services::private_store::atomic_write(path, &bytes)
}

pub fn upsert_source(
    registry: &mut AgentImportRegistry,
    selection: SourceSelection,
) -> Result<(), String> {
    validate_selection(&selection)?;
    if let Some(existing) = registry
        .sources
        .iter_mut()
        .find(|entry| entry.source_id == selection.source_id)
    {
        *existing = selection;
        return Ok(());
    }
    if registry.sources.len() >= MAX_SOURCES {
        return Err("Trop de sources configurées".into());
    }
    registry.sources.push(selection);
    Ok(())
}

fn validate_registry(registry: &AgentImportRegistry) -> Result<(), String> {
    if registry.version != REGISTRY_VERSION
        || registry.sources.len() > MAX_SOURCES
        || registry.documents.len() > 3
    {
        return Err("Configuration invalide".into());
    }
    let mut source_ids = HashSet::with_capacity(registry.sources.len());
    for source in &registry.sources {
        validate_selection(source)?;
        if !source_ids.insert(source.source_id.as_str()) {
            return Err("Configuration invalide".into());
        }
    }
    let mut document_names = HashSet::with_capacity(registry.documents.len());
    for document in &registry.documents {
        if !SOURCE_IDS.contains(&document.source_id.as_str())
            || !matches!(
                document.name.as_str(),
                "AGENTS.md" | "CLAUDE.md" | "QWEN.md"
            )
            || !document_names.insert(document.name.as_str())
            || document.source_hash.len() != 64
            || !document
                .source_hash
                .bytes()
                .all(|value| value.is_ascii_hexdigit())
            || document.source_path.as_os_str().len() > MAX_PATH_BYTES
            || !document.source_path.is_absolute()
            || document.source_path.to_string_lossy().contains('\0')
            || document
                .source_path
                .components()
                .any(|part| matches!(part, std::path::Component::ParentDir))
        {
            return Err("Configuration invalide".into());
        }
    }
    Ok(())
}

fn validate_selection(selection: &SourceSelection) -> Result<(), String> {
    if !SOURCE_IDS.contains(&selection.source_id.as_str())
        || selection.selected_skill_ids.len() > MAX_SKILLS_PER_SOURCE
        || selection.selected_rule_ids.len() > MAX_RULES_PER_SOURCE
        || selection.selected_document_ids.len() > 3
    {
        return Err("Sélection invalide".into());
    }
    let mut unique_ids = HashSet::new();
    for id in selection
        .selected_skill_ids
        .iter()
        .chain(&selection.selected_rule_ids)
        .chain(&selection.selected_document_ids)
    {
        if id.is_empty()
            || id.len() > MAX_ITEM_ID_BYTES
            || id.contains('\0')
            || id.contains("..")
            || !unique_ids.insert(id.as_str())
        {
            return Err("Sélection invalide".into());
        }
    }
    Ok(())
}

#[cfg(test)]
#[path = "registry_tests.rs"]
mod tests;
