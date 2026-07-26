use super::limits::{MAX_INSTRUCTION_BYTES, MAX_MANIFEST_BYTES};
use super::models::{DiscoveredItem, ImportItem, ImportItemKind};
use super::source_specs::SourceSpec;
use crate::services::agent_local::skill_parser::read_skill_metadata;
use sha2::{Digest, Sha256};
use std::io::ErrorKind;
use std::path::Path;

pub fn discover_documents(spec: &SourceSpec) -> (Vec<DiscoveredItem>, bool, bool) {
    let allowed_roots = spec
        .detection_roots
        .iter()
        .filter_map(|root| root.canonicalize().ok())
        .collect::<Vec<_>>();
    let mut items = Vec::new();
    let mut partial = false;
    let mut had_error = false;
    for candidate in &spec.documents {
        let metadata = match std::fs::metadata(&candidate.path) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == ErrorKind::NotFound => continue,
            Err(_) => {
                had_error = true;
                continue;
            }
        };
        if !metadata.is_file() || metadata.len() > MAX_INSTRUCTION_BYTES {
            partial = true;
            continue;
        }
        let Ok(canonical) = candidate.path.canonicalize() else {
            had_error = true;
            continue;
        };
        if !allowed_roots.iter().any(|root| canonical.starts_with(root)) {
            had_error = true;
            continue;
        }
        let mut item =
            plain_item_with_path(spec, &candidate.path, &canonical, ImportItemKind::Document);
        item.public.name = candidate.name.to_string();
        items.push(item);
    }
    (items, partial, had_error)
}

pub fn skill_item(spec: &SourceSpec, manifest: &Path) -> Option<DiscoveredItem> {
    let logical_bundle = manifest.parent()?.to_path_buf();
    let fallback = logical_bundle.file_name()?.to_string_lossy();
    let (name, description) = read_skill_metadata(manifest, &fallback, MAX_MANIFEST_BYTES)?;
    let canonical_manifest = manifest.canonicalize().ok()?;
    let canonical_bundle = canonical_manifest.parent()?.to_path_buf();
    Some(DiscoveredItem {
        public: import_item(
            spec,
            &logical_bundle,
            ImportItemKind::Skill,
            name,
            description,
        ),
        path: canonical_manifest,
        bundle_root: Some(canonical_bundle),
    })
}

pub fn plain_item_with_path(
    spec: &SourceSpec,
    logical_path: &Path,
    storage_path: &Path,
    kind: ImportItemKind,
) -> DiscoveredItem {
    let name = logical_path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("document")
        .to_string();
    DiscoveredItem {
        public: import_item(spec, logical_path, kind, name, String::new()),
        path: storage_path.to_path_buf(),
        bundle_root: None,
    }
}

fn import_item(
    spec: &SourceSpec,
    path: &Path,
    kind: ImportItemKind,
    name: String,
    description: String,
) -> ImportItem {
    ImportItem {
        id: item_id(spec.id, kind, path),
        name,
        description: description.chars().take(160).collect(),
        source_id: spec.id.to_string(),
        source_name: spec.display_name.to_string(),
        kind,
        selected: false,
        available: true,
        update_available: false,
    }
}

fn item_id(source_id: &str, kind: ImportItemKind, path: &Path) -> String {
    let kind = match kind {
        ImportItemKind::Document => "document",
        ImportItemKind::Rule => "rule",
        ImportItemKind::Skill => "skill",
    };
    let digest = Sha256::digest(path.to_string_lossy().as_bytes());
    format!("{source_id}:{kind}:{}", hex::encode(&digest[..12]))
}

pub fn public_items(items: &[DiscoveredItem]) -> Vec<ImportItem> {
    items.iter().map(|item| item.public.clone()).collect()
}
