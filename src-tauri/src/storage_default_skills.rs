use sha2::{Digest, Sha256};
use std::path::Path;
use tauri::Manager;

const MAX_DEFAULT_SKILLS: usize = 64;
const MAX_SKILL_MANIFEST_BYTES: u64 = 256 * 1024;
const LEGACY_SKILL_CREATE_SHA256: &str =
    "83bfadbb28ba109f15e2cef383ac8317c14c2358e388ae24dd6ab3b77e428dac";

pub(crate) struct ManagedSkillUpgrade<'a> {
    pub(crate) name: &'a str,
    pub(crate) legacy_manifest_sha256: &'a str,
}

const MANAGED_SKILL_UPGRADES: [ManagedSkillUpgrade<'static>; 1] = [ManagedSkillUpgrade {
    name: "skill-create",
    legacy_manifest_sha256: LEGACY_SKILL_CREATE_SHA256,
}];

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
        upgrade_manifest_if_untouched(&entry.path(), &target, upgrade)?;
    }
    Ok(())
}

fn upgrade_manifest_if_untouched(
    source_bundle: &Path,
    target_bundle: &Path,
    upgrade: &ManagedSkillUpgrade<'_>,
) -> Result<(), String> {
    let target = target_bundle.join("SKILL.md");
    let installed = match read_manifest(&target) {
        Ok(content) => content,
        Err(_) => return Ok(()),
    };
    if sha256_hex(&installed) != upgrade.legacy_manifest_sha256 {
        return Ok(());
    }

    let current = read_manifest(&source_bundle.join("SKILL.md"))?;
    crate::services::private_store::atomic_write(&target, &current)
        .map_err(|_| "Mise à jour du skill impossible".to_string())
}

fn read_manifest(path: &Path) -> Result<Vec<u8>, String> {
    let metadata =
        std::fs::symlink_metadata(path).map_err(|_| "Manifeste de skill indisponible")?;
    if !metadata.is_file()
        || metadata.file_type().is_symlink()
        || metadata.len() > MAX_SKILL_MANIFEST_BYTES
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
