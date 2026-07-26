use serde::Deserialize;
use std::collections::BTreeMap;
use std::path::{Component, Path};

use super::init_base_structure;

const PROFILE_MANIFEST: &str = include_str!("../../scripts/migration/cl-go-v1.0.2-profile.json");
const MAX_DOMAINS: usize = 32;
const MAX_PROFILE_FILES: usize = 96;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ProfileManifest {
    domains: Vec<ProfileDomain>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ProfileDomain {
    id: String,
    profile_paths: Vec<String>,
}

fn profile_files() -> Vec<(String, String)> {
    let manifest: ProfileManifest = serde_json::from_str(PROFILE_MANIFEST).unwrap();
    assert!(manifest.domains.len() <= MAX_DOMAINS);
    let mut unique = BTreeMap::new();
    for domain in manifest.domains {
        assert!(!domain.id.is_empty() && domain.id.len() <= 64);
        for path in domain.profile_paths {
            assert!(safe_relative_path(&path), "unsafe profile path");
            unique.entry(path).or_insert_with(|| domain.id.clone());
            assert!(unique.len() <= MAX_PROFILE_FILES);
        }
    }
    unique.into_iter().collect()
}

fn safe_relative_path(value: &str) -> bool {
    let path = Path::new(value);
    !value.is_empty()
        && value.len() <= 512
        && !path.is_absolute()
        && path
            .components()
            .all(|component| matches!(component, Component::Normal(_)))
}

fn seed_profile(root: &Path, files: &[(String, String)]) {
    for (path, domain) in files {
        let target = root.join(path);
        std::fs::create_dir_all(target.parent().unwrap()).unwrap();
        let marker = format!("cl-go-v1.0.2:{domain}:{path}");
        std::fs::write(target, marker).unwrap();
    }
}

fn assert_profile(root: &Path, files: &[(String, String)]) {
    for (path, domain) in files {
        let marker = format!("cl-go-v1.0.2:{domain}:{path}");
        assert_eq!(std::fs::read(root.join(path)).unwrap(), marker.as_bytes());
    }
}

#[test]
fn beaver_initialization_preserves_profile_and_rollback_copy() {
    let temp = tempfile::tempdir().unwrap();
    let primary = temp.path().join("cl-go-dash");
    let rollback = temp.path().join("cl-go-dash-rollback");
    let files = profile_files();
    std::fs::create_dir_all(&primary).unwrap();
    seed_profile(&primary, &files);

    crate::storage_migration_files::copy_recursive(&primary, &rollback).unwrap();
    init_base_structure(&primary).unwrap();

    assert_profile(&primary, &files);
    assert_profile(&rollback, &files);
    assert!(!primary.join("beaver").exists());
    assert!(!primary.join(".local/share/beaver").exists());
}
