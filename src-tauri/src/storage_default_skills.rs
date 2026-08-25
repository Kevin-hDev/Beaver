use sha2::{Digest, Sha256};
use std::path::Path;
use tauri::Manager;

use crate::services::skill_manifest_policy::MAX_SKILL_MANIFEST_BYTES;

const MAX_DEFAULT_SKILLS: usize = 64;
const MAX_SKILL_BUNDLE_ENTRIES: usize = 128;
const LEGACY_SKILL_CREATE_SHA256: &str =
    "83bfadbb28ba109f15e2cef383ac8317c14c2358e388ae24dd6ab3b77e428dac";
const LEGACY_FORECASTING_STUB_SHA256: &str =
    "a9c22c6ee64ab25be3b74b08455b7de48d7f403275127dd7ced1f026d56e46e6";
const LEGACY_FORECAST_MODEL_ROUTER_STUB_SHA256: &str =
    "2f29a1e024e644d6e27c9637e0776f28f906f3bcae4ed8c0844d3d0f4200e54a";

#[path = "storage_default_skills_migration.rs"]
mod migration;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ManagedSkillUpgradeKind {
    ManifestOnly,
    FullBundle,
}

pub(crate) struct ManagedSkillUpgrade<'a> {
    pub(crate) name: &'a str,
    pub(crate) legacy_manifest_sha256: &'a str,
    pub(crate) kind: ManagedSkillUpgradeKind,
}

const MANAGED_SKILL_UPGRADES: [ManagedSkillUpgrade<'static>; 3] = [
    ManagedSkillUpgrade {
        name: "skill-create",
        legacy_manifest_sha256: LEGACY_SKILL_CREATE_SHA256,
        kind: ManagedSkillUpgradeKind::ManifestOnly,
    },
    ManagedSkillUpgrade {
        name: "forecasting",
        legacy_manifest_sha256: LEGACY_FORECASTING_STUB_SHA256,
        kind: ManagedSkillUpgradeKind::FullBundle,
    },
    ManagedSkillUpgrade {
        name: "forecast-model-router",
        legacy_manifest_sha256: LEGACY_FORECAST_MODEL_ROUTER_STUB_SHA256,
        kind: ManagedSkillUpgradeKind::FullBundle,
    },
];

pub(crate) fn install_default_skills(
    app_handle: &tauri::AppHandle,
    base: &Path,
) -> Result<(), String> {
    let resource_base = app_handle
        .path()
        .resource_dir()
        .map_err(|_| "Skills intégrés indisponibles")?
        .join("default-skills");
    sync_default_skills_from(
        &resource_base,
        &base.join("skills"),
        &MANAGED_SKILL_UPGRADES,
    )
}

pub(crate) fn sync_default_skills_from(
    resource_base: &Path,
    skills_dir: &Path,
    upgrades: &[ManagedSkillUpgrade<'_>],
) -> Result<(), String> {
    migration::migrate_untouched_legacy_hk_debug(resource_base, skills_dir)?;
    let entries = std::fs::read_dir(resource_base).map_err(|_| "Skills intégrés indisponibles")?;
    let mut count = 0_usize;
    for entry in entries {
        count += 1;
        if count > MAX_DEFAULT_SKILLS {
            return Err("Trop de skills intégrés".to_string());
        }
        let entry = entry.map_err(|_| "Skills intégrés indisponibles")?;
        let file_type = entry
            .file_type()
            .map_err(|_| "Skills intégrés indisponibles")?;
        if file_type.is_symlink() {
            return Err("Ressource de skill invalide".to_string());
        }
        if !file_type.is_dir() {
            continue;
        }

        let target = skills_dir.join(entry.file_name());
        if !target.exists() {
            super::storage_migration_files::copy_recursive(&entry.path(), &target)
                .map_err(|_| "Installation des skills impossible")?;
            continue;
        }

        let Some(upgrade) = upgrades
            .iter()
            .find(|upgrade| entry.file_name() == upgrade.name)
        else {
            continue;
        };
        upgrade_if_untouched(&entry.path(), &target, upgrade)?;
    }
    Ok(())
}

fn upgrade_if_untouched(
    source_bundle: &Path,
    target_bundle: &Path,
    upgrade: &ManagedSkillUpgrade<'_>,
) -> Result<(), String> {
    let metadata = match std::fs::symlink_metadata(target_bundle) {
        Ok(metadata) => metadata,
        Err(_) => return Ok(()),
    };
    if !metadata.is_dir() || metadata.file_type().is_symlink() {
        return Ok(());
    }

    let target = target_bundle.join("SKILL.md");
    let installed = match read_manifest(&target) {
        Ok(content) => content,
        Err(_) => return Ok(()),
    };
    if sha256_hex(&installed) != upgrade.legacy_manifest_sha256 {
        return Ok(());
    }

    match upgrade.kind {
        ManagedSkillUpgradeKind::ManifestOnly => {
            let current = read_manifest(&source_bundle.join("SKILL.md"))?;
            crate::services::private_store::atomic_write(&target, &current)
                .map_err(|_| "Mise à jour du skill impossible".to_string())
        }
        ManagedSkillUpgradeKind::FullBundle => {
            sync_full_bundle_manifest_last(source_bundle, target_bundle)
        }
    }
}

fn sync_full_bundle_manifest_last(
    source_bundle: &Path,
    target_bundle: &Path,
) -> Result<(), String> {
    let entries = std::fs::read_dir(source_bundle).map_err(|_| skill_upgrade_error())?;
    let mut count = 0_usize;
    for entry in entries {
        let entry = entry.map_err(|_| skill_upgrade_error())?;
        if entry.file_name() == "SKILL.md" {
            continue;
        }
        count += 1;
        if count > MAX_SKILL_BUNDLE_ENTRIES {
            return Err(skill_upgrade_error());
        }
        super::storage_migration_files::copy_recursive(
            &entry.path(),
            &target_bundle.join(entry.file_name()),
        )
        .map_err(|_| skill_upgrade_error())?;
    }

    let current = read_manifest(&source_bundle.join("SKILL.md"))?;
    crate::services::private_store::atomic_write(&target_bundle.join("SKILL.md"), &current)
        .map_err(|_| skill_upgrade_error())
}

fn skill_upgrade_error() -> String {
    "Mise à jour du skill impossible".to_string()
}

fn read_manifest(path: &Path) -> Result<Vec<u8>, String> {
    let metadata =
        std::fs::symlink_metadata(path).map_err(|_| "Manifeste de skill indisponible")?;
    if !metadata.is_file()
        || metadata.file_type().is_symlink()
        || metadata.len() > MAX_SKILL_MANIFEST_BYTES as u64
    {
        return Err("Manifeste de skill invalide".to_string());
    }
    std::fs::read(path).map_err(|_| "Manifeste de skill indisponible".to_string())
}

fn sha256_hex(content: &[u8]) -> String {
    hex::encode(Sha256::digest(content))
}

#[cfg(test)]
#[path = "storage_default_skills_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "storage_default_skills_upgrade_tests.rs"]
mod upgrade_tests;
