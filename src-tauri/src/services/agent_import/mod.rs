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

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

pub const MAX_ENABLED_RESOURCE_DIRS: usize = limits::MAX_SOURCES * limits::MAX_ROOTS_PER_SOURCE * 2;
pub const MAX_ENABLED_RESOURCE_FILES: usize = limits::MAX_SOURCES * 3;

#[derive(Default)]
pub struct EnabledResourcePaths {
    pub directories: Vec<PathBuf>,
    pub files: Vec<PathBuf>,
}

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

pub fn enabled_resource_paths(home: &Path) -> EnabledResourcePaths {
    enabled_resource_paths_from(home, &registry::read())
}

fn enabled_resource_paths_from(
    home: &Path,
    registry: &registry::AgentImportRegistry,
) -> EnabledResourcePaths {
    let private_store = crate::services::paths::data_dir();
    let private_store = dunce::canonicalize(&private_store).unwrap_or(private_store);
    let mut directories = BTreeSet::new();
    let mut files = BTreeSet::new();
    for spec in source_specs::source_specs(home).into_iter().filter(|spec| {
        registry
            .sources
            .iter()
            .any(|source| source.source_id == spec.id && source.enabled)
    }) {
        let detection_roots = spec
            .detection_roots
            .iter()
            .filter(|root| !is_symlink(root))
            .filter_map(|root| dunce::canonicalize(root).ok())
            .collect::<Vec<_>>();
        for path in spec.rule_roots.iter().chain(&spec.skill_roots) {
            let Some(path) = canonical_resource(path, true, &detection_roots, &private_store)
            else {
                continue;
            };
            if directories.len() < MAX_ENABLED_RESOURCE_DIRS {
                directories.insert(path);
            }
        }
        for document in &spec.documents {
            let Some(path) =
                canonical_resource(&document.path, false, &detection_roots, &private_store)
            else {
                continue;
            };
            if files.len() < MAX_ENABLED_RESOURCE_FILES {
                files.insert(path);
            }
        }
    }
    EnabledResourcePaths {
        directories: directories.into_iter().collect(),
        files: files.into_iter().collect(),
    }
}

fn canonical_resource(
    path: &Path,
    directory: bool,
    detection_roots: &[PathBuf],
    private_store: &Path,
) -> Option<PathBuf> {
    if is_symlink(path) {
        return None;
    }
    let canonical = dunce::canonicalize(path).ok()?;
    if is_symlink(path) || dunce::canonicalize(path).ok().as_ref() != Some(&canonical) {
        return None;
    }
    let expected_kind = if directory {
        canonical.is_dir()
    } else {
        canonical.is_file()
    };
    (expected_kind
        && detection_roots
            .iter()
            .any(|root| canonical.starts_with(root))
        && !canonical.starts_with(private_store)
        && !private_store.starts_with(&canonical))
    .then_some(canonical)
}

fn is_symlink(path: &Path) -> bool {
    path.symlink_metadata()
        .is_ok_and(|metadata| metadata.file_type().is_symlink())
}

#[cfg(test)]
pub(crate) fn declared_resource_counts(home: &Path) -> (usize, usize) {
    let specs = source_specs::source_specs(home);
    let directories = specs
        .iter()
        .flat_map(|spec| spec.rule_roots.iter().chain(&spec.skill_roots))
        .collect::<BTreeSet<_>>()
        .len();
    let files = specs
        .iter()
        .flat_map(|spec| spec.documents.iter().map(|document| &document.path))
        .collect::<BTreeSet<_>>()
        .len();
    (directories, files)
}

#[cfg(test)]
#[path = "integration_tests.rs"]
mod integration_tests;
