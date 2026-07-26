use super::discovery_items::{discover_documents, plain_item_with_path, public_items, skill_item};
use super::limits::MAX_TOTAL_SKILLS;
use super::models::{
    AgentSourceSummary, DiscoveredItem, DiscoveredSource, ImportItemKind, SelectionMode,
    SourceStatus,
};
use super::registry::AgentImportRegistry;
use super::rule_walker::find_rules;
use super::source_specs::{source_specs, SourceSpec};
use super::walker::find_skills;
use std::path::Path;

pub fn scan_sources(home: &Path, registry: &AgentImportRegistry) -> Vec<DiscoveredSource> {
    let mut sources = Vec::new();
    let mut total_skills = 0_usize;
    for spec in source_specs(home) {
        let mut source = scan_source(&spec, home, registry);
        let remaining = MAX_TOTAL_SKILLS.saturating_sub(total_skills);
        if source.skills.len() > remaining {
            source.summary.partial = true;
        }
        source.skills.truncate(remaining);
        source.summary.skills.truncate(remaining);
        total_skills += source.skills.len();
        sources.push(source);
    }
    sources
}

pub fn scan_source(
    spec: &SourceSpec,
    home: &Path,
    registry: &AgentImportRegistry,
) -> DiscoveredSource {
    let stored = registry
        .sources
        .iter()
        .find(|entry| entry.source_id == spec.id);
    let (mut documents, document_partial, document_error) = discover_documents(spec);
    mark_document_updates(&mut documents, registry);
    let rule_walk = find_rules(&spec.rule_roots, home);
    let rules = rule_walk
        .files
        .iter()
        .filter_map(|path| {
            let canonical = path.canonicalize().ok()?;
            Some(plain_item_with_path(
                spec,
                path,
                &canonical,
                ImportItemKind::Rule,
            ))
        })
        .collect::<Vec<_>>();
    let skill_walk = find_skills(&spec.skill_roots, home);
    let skills = skill_walk
        .files
        .iter()
        .filter_map(|path| skill_item(spec, path))
        .collect::<Vec<_>>();
    let detected = spec.detection_roots.iter().any(|root| root.exists());
    let item_count = documents.len() + rules.len() + skills.len();
    let partial = document_partial
        || document_error
        || rule_walk.partial
        || skill_walk.partial
        || rule_walk.had_error
        || skill_walk.had_error
        || rules.len() != rule_walk.files.len()
        || skills.len() != skill_walk.files.len();
    let had_error = document_error
        || rule_walk.had_error
        || skill_walk.had_error
        || rules.len() != rule_walk.files.len();
    let status = match (detected, item_count, had_error) {
        (false, _, _) => SourceStatus::Missing,
        (true, 0, true) => SourceStatus::Unavailable,
        (true, 0, false) => SourceStatus::Empty,
        (true, _, _) => SourceStatus::Detected,
    };
    let configured = stored.is_some();
    let enabled = stored.is_some_and(|entry| entry.enabled);
    let documents = select_items(documents, stored, ImportItemKind::Document);
    let rules = select_items(rules, stored, ImportItemKind::Rule);
    let skills = select_items(skills, stored, ImportItemKind::Skill);
    let summary = AgentSourceSummary {
        id: spec.id.to_string(),
        display_name: spec.display_name.to_string(),
        status,
        partial,
        configured,
        enabled,
        documents: public_items(&documents),
        rules: public_items(&rules),
        skills: public_items(&skills),
    };
    DiscoveredSource {
        summary,
        documents,
        rules,
        skills,
    }
}

fn mark_document_updates(documents: &mut [DiscoveredItem], registry: &AgentImportRegistry) {
    for item in documents {
        let Some(stored) = registry.documents.iter().find(|document| {
            document.source_id == item.public.source_id && document.name == item.public.name
        }) else {
            continue;
        };
        item.public.update_available =
            !super::document_io::file_hash_matches(&item.path, &stored.source_hash);
    }
}

fn select_items(
    mut items: Vec<DiscoveredItem>,
    stored: Option<&super::models::SourceSelection>,
    kind: ImportItemKind,
) -> Vec<DiscoveredItem> {
    for item in &mut items {
        item.public.selected = match (stored, kind) {
            (None, _) => true,
            (Some(value), ImportItemKind::Skill) => match value.skill_mode {
                SelectionMode::All => true,
                SelectionMode::None => false,
                SelectionMode::Custom => value.selected_skill_ids.contains(&item.public.id),
            },
            (Some(value), ImportItemKind::Rule) => {
                value.selected_rule_ids.contains(&item.public.id)
            }
            (Some(value), ImportItemKind::Document) => {
                value.selected_document_ids.contains(&item.public.id)
            }
        };
    }
    items.sort_by(|left, right| {
        left.public
            .name
            .to_lowercase()
            .cmp(&right.public.name.to_lowercase())
            .then(left.public.id.cmp(&right.public.id))
    });
    items
}

#[cfg(test)]
#[path = "discovery_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "discovery_security_tests.rs"]
mod security_tests;
