use std::path::Path;

const LEGACY_HK_DEBUG_SHA256: &str =
    "d528c33dc6ad9253e161c5ade4e5a8d064245d0f11eb90a996043a2a4239e192";

pub(super) fn migrate_untouched_legacy_hk_debug(
    resource_base: &Path,
    skills_dir: &Path,
) -> Result<(), String> {
    migrate_legacy_bundle(resource_base, skills_dir, LEGACY_HK_DEBUG_SHA256)
}

#[cfg(test)]
pub(super) fn migrate_matching_legacy_hk_debug(
    resource_base: &Path,
    skills_dir: &Path,
    legacy_manifest_sha256: &str,
) -> Result<(), String> {
    migrate_legacy_bundle(resource_base, skills_dir, legacy_manifest_sha256)
}

fn migrate_legacy_bundle(
    resource_base: &Path,
    skills_dir: &Path,
    legacy_manifest_sha256: &str,
) -> Result<(), String> {
    let legacy_bundle = skills_dir.join("hk-debug");
    let current_bundle = skills_dir.join("root-cause-debugging");
    if current_bundle.exists() {
        return Ok(());
    }

    let metadata = match std::fs::symlink_metadata(&legacy_bundle) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(_) => return Err(super::skill_upgrade_error()),
    };
    if !metadata.is_dir() || metadata.file_type().is_symlink() {
        return Ok(());
    }

    let legacy_manifest = match super::read_manifest(&legacy_bundle.join("SKILL.md")) {
        Ok(content) => content,
        Err(_) => return Ok(()),
    };
    if super::sha256_hex(&legacy_manifest) != legacy_manifest_sha256 {
        return Ok(());
    }

    let current_manifest =
        super::read_manifest(&resource_base.join("root-cause-debugging").join("SKILL.md"))?;
    std::fs::rename(&legacy_bundle, &current_bundle).map_err(|_| super::skill_upgrade_error())?;
    if crate::services::private_store::atomic_write(
        &current_bundle.join("SKILL.md"),
        &current_manifest,
    )
    .is_err()
    {
        let _ = std::fs::rename(&current_bundle, &legacy_bundle);
        return Err(super::skill_upgrade_error());
    }
    Ok(())
}
