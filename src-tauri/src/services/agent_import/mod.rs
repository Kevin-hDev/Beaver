mod discovery;
mod discovery_items;
mod document_io;
mod documents;
mod limits;
mod models;
mod registry;
mod rule_content;
mod rule_walker;
mod source_paths;
mod source_specs;
mod walker;

pub use documents::save_source_selection;
use models::{DiscoveredItem, DiscoveredSource};
pub(crate) use rule_content::{selected_rule_contents, ExternalRuleContent};

pub use models::{AgentSourceSummary, SaveSelectionResult, SourceSelection};

use std::path::Path;

pub fn scan_from(home: &Path) -> Vec<DiscoveredSource> {
    let registry = registry::read();
    discovery::scan_sources(home, &registry)
}

pub fn public_sources(home: &Path) -> Vec<AgentSourceSummary> {
    scan_from(home)
        .into_iter()
        .map(|source| source.summary)
        .collect()
}

pub fn selected_skills(home: &Path) -> Vec<DiscoveredItem> {
    selected_sources(home)
        .into_iter()
        .flat_map(|source| source.skills)
        .filter(|item| item.public.selected && item.public.available)
        .collect()
}

fn selected_rules(home: &Path) -> Vec<DiscoveredItem> {
    selected_sources(home)
        .into_iter()
        .flat_map(|source| source.rules)
        .filter(|item| item.public.selected && item.public.available)
        .collect()
}

fn selected_sources(home: &Path) -> Vec<DiscoveredSource> {
    let registry = registry::read();
    selected_sources_from(home, &registry)
}

fn selected_sources_from(
    home: &Path,
    registry: &registry::AgentImportRegistry,
) -> Vec<DiscoveredSource> {
    source_specs::source_specs(home)
        .into_iter()
        .filter(|spec| {
            registry
                .sources
                .iter()
                .any(|source| source.source_id == spec.id && source.enabled)
        })
        .map(|spec| discovery::scan_source(&spec, home, registry))
        .collect()
}

pub fn selected_skill_roots(home: &Path) -> Vec<std::path::PathBuf> {
    selected_skills(home)
        .into_iter()
        .filter_map(|item| item.bundle_root)
        .filter_map(|path| path.canonicalize().ok())
        .collect()
}

pub fn enabled_hidden_documents(data_dir: &Path) -> Vec<String> {
    let registry = registry::read_from(&data_dir.join("external-agent-sources.json"));
    registry
        .documents
        .into_iter()
        .filter(|document| {
            document.enabled && matches!(document.name.as_str(), "CLAUDE.md" | "QWEN.md")
        })
        .map(|document| document.name)
        .collect()
}

#[cfg(test)]
#[path = "integration_tests.rs"]
mod integration_tests;
