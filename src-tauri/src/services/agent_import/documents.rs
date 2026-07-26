use super::discovery::scan_source;
use super::document_io::{backup_document, read_instruction, same_file_content};
use super::models::{SaveSelectionResult, SelectionMode, SourceSelection};
use super::registry::{self, AgentImportRegistry, ImportedDocument};
use super::source_specs::source_specs;
use sha2::{Digest, Sha256};
use std::path::Path;

pub fn save_source_selection(
    home: &Path,
    selection: SourceSelection,
    replace_documents: bool,
) -> Result<SaveSelectionResult, String> {
    let data_dir = crate::services::paths::data_dir();
    save_source_selection_to(home, &data_dir, selection, replace_documents)
}

pub(crate) fn save_source_selection_to(
    home: &Path,
    data_dir: &Path,
    mut selection: SourceSelection,
    replace_documents: bool,
) -> Result<SaveSelectionResult, String> {
    let registry_path = data_dir.join("external-agent-sources.json");
    let mut registry = registry::read_from(&registry_path);
    preserve_imported_documents(&registry, &mut selection);
    if !selection.enabled {
        disable_source(&mut registry, selection)?;
        registry::write_to(&registry_path, &registry)?;
        return Ok(SaveSelectionResult {
            saved: true,
            conflicts: Vec::new(),
        });
    }
    registry::upsert_source(&mut registry, selection.clone())?;
    let spec = source_specs(home)
        .into_iter()
        .find(|spec| spec.id == selection.source_id)
        .ok_or_else(|| "Source invalide".to_string())?;
    let discovered = scan_source(&spec, home, &registry);
    validate_selected_ids(&selection, &discovered)?;
    let selected_documents = discovered
        .documents
        .iter()
        .filter(|item| selection.selected_document_ids.contains(&item.public.id))
        .collect::<Vec<_>>();
    let conflicts = document_conflicts(data_dir, &selected_documents)?;
    if !conflicts.is_empty() && !replace_documents {
        return Ok(SaveSelectionResult {
            saved: false,
            conflicts,
        });
    }
    for item in selected_documents {
        let bytes = read_instruction(&item.path)?;
        let destination = data_dir.join(&item.public.name);
        if destination.exists() && !same_file_content(&destination, &bytes)? {
            let existing = std::fs::metadata(&destination)
                .map(|metadata| metadata.len())
                .unwrap_or_default();
            if existing > 0 {
                backup_document(data_dir, &destination)?;
            }
        }
        crate::services::private_store::atomic_write(&destination, &bytes)?;
        upsert_document(
            &mut registry,
            &item.public.name,
            &selection.source_id,
            &item.path,
            &bytes,
        );
    }
    registry::write_to(&registry_path, &registry)?;
    Ok(SaveSelectionResult {
        saved: true,
        conflicts: Vec::new(),
    })
}

fn preserve_imported_documents(registry: &AgentImportRegistry, selection: &mut SourceSelection) {
    let Some(stored) = registry
        .sources
        .iter()
        .find(|source| source.source_id == selection.source_id)
    else {
        return;
    };
    for id in &stored.selected_document_ids {
        if !selection.selected_document_ids.contains(id) {
            selection.selected_document_ids.push(id.clone());
        }
    }
}

fn disable_source(
    registry: &mut AgentImportRegistry,
    selection: SourceSelection,
) -> Result<(), String> {
    let source_id = selection.source_id.clone();
    if let Some(stored) = registry
        .sources
        .iter_mut()
        .find(|source| source.source_id == source_id)
    {
        stored.enabled = false;
    } else {
        registry::upsert_source(registry, selection)?;
    }
    Ok(())
}

fn validate_selected_ids(
    selection: &SourceSelection,
    source: &super::models::DiscoveredSource,
) -> Result<(), String> {
    let valid_skills = source
        .skills
        .iter()
        .map(|item| item.public.id.as_str())
        .collect::<Vec<_>>();
    if selection.skill_mode == SelectionMode::Custom
        && selection
            .selected_skill_ids
            .iter()
            .any(|id| !valid_skills.contains(&id.as_str()))
    {
        return Err("Sélection invalide".into());
    }
    let valid_rules = source
        .rules
        .iter()
        .map(|item| item.public.id.as_str())
        .collect::<Vec<_>>();
    let valid_documents = source
        .documents
        .iter()
        .map(|item| item.public.id.as_str())
        .collect::<Vec<_>>();
    if selection
        .selected_rule_ids
        .iter()
        .any(|id| !valid_rules.contains(&id.as_str()))
        || selection
            .selected_document_ids
            .iter()
            .any(|id| !valid_documents.contains(&id.as_str()))
    {
        return Err("Sélection invalide".into());
    }
    Ok(())
}

fn document_conflicts(
    data_dir: &Path,
    documents: &[&super::models::DiscoveredItem],
) -> Result<Vec<String>, String> {
    let mut conflicts = Vec::new();
    for item in documents {
        let destination = data_dir.join(&item.public.name);
        let bytes = read_instruction(&item.path)?;
        if destination.exists() && !same_file_content(&destination, &bytes)? {
            let existing = std::fs::metadata(&destination)
                .map(|metadata| metadata.len())
                .unwrap_or_default();
            if existing > 0 {
                conflicts.push(item.public.name.clone());
            }
        }
    }
    Ok(conflicts)
}

fn upsert_document(
    registry: &mut AgentImportRegistry,
    name: &str,
    source_id: &str,
    source_path: &Path,
    bytes: &[u8],
) {
    registry.documents.retain(|entry| entry.name != name);
    registry.documents.push(ImportedDocument {
        name: name.to_string(),
        source_id: source_id.to_string(),
        source_path: source_path.to_path_buf(),
        source_hash: hex::encode(Sha256::digest(bytes)),
        enabled: true,
    });
}

#[cfg(test)]
#[path = "documents_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "documents_persistence_tests.rs"]
mod persistence_tests;
