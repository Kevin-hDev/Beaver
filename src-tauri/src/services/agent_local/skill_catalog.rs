use super::skill_parser::{read_skill_metadata, MAX_SKILL_DESCRIPTION_CHARS};
use super::types_tools::SkillInfo;
use crate::services::agent_import;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, HashMap};
use std::path::{Path, PathBuf};

const SKILL_FILENAMES: &[&str] = &["skill.md", "SKILL.md"];
const MAX_LOCAL_SKILLS: usize = 2048;

pub struct SkillCatalogEntry {
    pub info: SkillInfo,
    pub manifest: PathBuf,
    pub bundle_root: PathBuf,
}

pub fn entries() -> Result<Vec<SkillCatalogEntry>, String> {
    let mut entries = local_entries()?;
    if let Some(home) = dirs::home_dir() {
        entries.extend(external_entries(&home));
    }
    entries.sort_by(|left, right| {
        left.info
            .name
            .to_lowercase()
            .cmp(&right.info.name.to_lowercase())
            .then(left.info.id.cmp(&right.info.id))
    });
    make_commands_unique(&mut entries);
    Ok(entries)
}

fn local_entries() -> Result<Vec<SkillCatalogEntry>, String> {
    let root = crate::services::paths::data_dir().join("skills");
    if !root.exists() {
        return Ok(Vec::new());
    }
    let read_dir = std::fs::read_dir(&root).map_err(|_| "Skills indisponibles")?;
    let mut bundles = BTreeMap::new();
    for entry in read_dir.filter_map(Result::ok) {
        bundles.insert(entry.file_name(), entry.path());
        if bundles.len() > MAX_LOCAL_SKILLS {
            bundles.pop_last();
        }
    }
    let mut entries = Vec::new();
    for (fallback, bundle) in bundles {
        if !bundle.is_dir() {
            continue;
        }
        let Some(manifest) = find_skill_file(&bundle) else {
            continue;
        };
        let Some((name, description)) = metadata(&manifest, &fallback.to_string_lossy()) else {
            continue;
        };
        let id = catalog_id("local", &bundle);
        entries.push(SkillCatalogEntry {
            info: SkillInfo {
                command: command_name("local", &name, &id),
                path: id.clone(),
                id,
                name,
                description,
                source: "local".to_string(),
                source_name: local_source_name().to_string(),
            },
            manifest,
            bundle_root: bundle,
        });
    }
    Ok(entries)
}

fn local_source_name() -> &'static str {
    crate::services::brand::DISPLAY_NAME
}

fn external_entries(home: &Path) -> Vec<SkillCatalogEntry> {
    agent_import::selected_skills(home)
        .into_iter()
        .filter_map(|item| {
            let bundle = item.bundle_root?;
            let id = item.public.id;
            let name = item.public.name;
            Some(SkillCatalogEntry {
                info: SkillInfo {
                    command: command_name(&item.public.source_id, &name, &id),
                    path: id.clone(),
                    id,
                    name,
                    description: item.public.description,
                    source: item.public.source_id,
                    source_name: item.public.source_name,
                },
                manifest: item.path,
                bundle_root: bundle,
            })
        })
        .collect()
}

fn metadata(manifest: &Path, fallback: &str) -> Option<(String, String)> {
    let (name, description) = read_skill_metadata(
        manifest,
        fallback,
        super::skill_limits::MAX_SKILL_CONTENT_BYTES as u64,
    )?;
    Some((
        name,
        description
            .chars()
            .take(MAX_SKILL_DESCRIPTION_CHARS)
            .collect(),
    ))
}

fn find_skill_file(directory: &Path) -> Option<PathBuf> {
    SKILL_FILENAMES.iter().find_map(|name| {
        let path = directory.join(name);
        path.is_file().then_some(path)
    })
}

fn catalog_id(source: &str, path: &Path) -> String {
    let canonical = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    let digest = Sha256::digest(canonical.to_string_lossy().as_bytes());
    format!("{source}:skill:{}", hex::encode(&digest[..12]))
}

fn command_name(source: &str, name: &str, id: &str) -> String {
    let clean = name
        .chars()
        .filter(|value| value.is_alphanumeric() || matches!(value, '-' | '_' | ':'))
        .take(80)
        .collect::<String>();
    if clean.is_empty() {
        format!("{source}:{}", &id[id.len().saturating_sub(12)..])
    } else if source == "local" {
        clean
    } else {
        format!("{source}:{clean}")
    }
}

fn make_commands_unique(entries: &mut [SkillCatalogEntry]) {
    let mut counts = HashMap::new();
    for entry in entries.iter() {
        *counts.entry(entry.info.command.clone()).or_insert(0_usize) += 1;
    }
    for entry in entries {
        if counts.get(&entry.info.command).copied().unwrap_or_default() > 1 {
            let suffix = &entry.info.id[entry.info.id.len().saturating_sub(8)..];
            entry.info.command = format!("{}:{suffix}", entry.info.command);
        }
    }
}

#[cfg(test)]
#[path = "skill_catalog_tests.rs"]
mod tests;
