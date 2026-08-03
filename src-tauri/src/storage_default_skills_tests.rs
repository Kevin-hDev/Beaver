use super::{
    migration::migrate_matching_legacy_hk_debug, sha256_hex, sync_default_skills_from,
    ManagedSkillUpgrade, ManagedSkillUpgradeKind, LEGACY_SKILL_CREATE_SHA256,
};

fn write_skill(root: &std::path::Path, content: &[u8]) {
    let bundle = root.join("skill-create");
    std::fs::create_dir_all(&bundle).unwrap();
    std::fs::write(bundle.join("SKILL.md"), content).unwrap();
}

fn upgrade_for<'a>(hash: &'a str) -> ManagedSkillUpgrade<'a> {
    ManagedSkillUpgrade {
        name: "skill-create",
        legacy_manifest_sha256: hash,
        kind: ManagedSkillUpgradeKind::ManifestOnly,
    }
}

#[test]
fn installs_the_current_default_skill_when_missing() {
    let temp = tempfile::tempdir().unwrap();
    let resources = temp.path().join("resources");
    let installed = temp.path().join("installed");
    write_skill(&resources, b"current");

    sync_default_skills_from(&resources, &installed, &[]).unwrap();

    assert_eq!(
        std::fs::read(installed.join("skill-create/SKILL.md")).unwrap(),
        b"current"
    );
}

#[test]
fn upgrades_the_untouched_legacy_skill_and_preserves_extra_files() {
    let temp = tempfile::tempdir().unwrap();
    let resources = temp.path().join("resources");
    let installed = temp.path().join("installed");
    write_skill(&resources, b"current");
    write_skill(&installed, b"official-v1");
    std::fs::write(installed.join("skill-create/user-note.md"), b"keep").unwrap();
    let legacy_hash = sha256_hex(b"official-v1");
    let upgrades = [upgrade_for(&legacy_hash)];

    sync_default_skills_from(&resources, &installed, &upgrades).unwrap();

    assert_eq!(
        std::fs::read(installed.join("skill-create/SKILL.md")).unwrap(),
        b"current"
    );
    assert_eq!(
        std::fs::read(installed.join("skill-create/user-note.md")).unwrap(),
        b"keep"
    );
}

#[test]
fn preserves_a_personalized_legacy_skill() {
    let temp = tempfile::tempdir().unwrap();
    let resources = temp.path().join("resources");
    let installed = temp.path().join("installed");
    write_skill(&resources, b"current");
    write_skill(&installed, b"personalized");
    let legacy_hash = sha256_hex(b"official-v1");
    let upgrades = [upgrade_for(&legacy_hash)];

    sync_default_skills_from(&resources, &installed, &upgrades).unwrap();

    assert_eq!(
        std::fs::read(installed.join("skill-create/SKILL.md")).unwrap(),
        b"personalized"
    );
}

#[test]
fn preserves_an_existing_bundle_without_a_manifest() {
    let temp = tempfile::tempdir().unwrap();
    let resources = temp.path().join("resources");
    let installed = temp.path().join("installed");
    write_skill(&resources, b"current");
    std::fs::create_dir_all(installed.join("skill-create")).unwrap();
    std::fs::write(installed.join("skill-create/user-note.md"), b"keep").unwrap();
    let legacy_hash = sha256_hex(b"official-v1");
    let upgrades = [upgrade_for(&legacy_hash)];

    sync_default_skills_from(&resources, &installed, &upgrades).unwrap();

    assert!(!installed.join("skill-create/SKILL.md").exists());
    assert_eq!(
        std::fs::read(installed.join("skill-create/user-note.md")).unwrap(),
        b"keep"
    );
}

#[test]
fn bundled_skill_create_uses_the_cl_go_dash_destination_contract() {
    let content = include_str!("../default-skills/skill-create/SKILL.md");

    assert!(content.contains("~/.local/share/cl-go-dash/skills"));
    assert!(content.contains("You never infer the destination from imported skills"));
    assert!(!content.contains("CLAUDE_SKILL_DIR"));
    assert_ne!(sha256_hex(content.as_bytes()), LEGACY_SKILL_CREATE_SHA256);
}

#[test]
fn renames_the_untouched_hk_debug_bundle_and_preserves_extra_files() {
    let temp = tempfile::tempdir().unwrap();
    let resources = temp.path().join("resources");
    let installed = temp.path().join("installed");
    let legacy = b"official hk-debug";
    let current = b"official root-cause-debugging";
    let legacy_hash = sha256_hex(legacy);
    let resource_bundle = resources.join("root-cause-debugging");
    let installed_bundle = installed.join("hk-debug");
    std::fs::create_dir_all(&resource_bundle).unwrap();
    std::fs::create_dir_all(&installed_bundle).unwrap();
    std::fs::write(resource_bundle.join("SKILL.md"), current).unwrap();
    std::fs::write(installed_bundle.join("SKILL.md"), legacy).unwrap();
    std::fs::write(installed_bundle.join("user-note.md"), b"keep").unwrap();

    migrate_matching_legacy_hk_debug(&resources, &installed, &legacy_hash).unwrap();

    assert!(!installed.join("hk-debug").exists());
    assert_eq!(
        std::fs::read(installed.join("root-cause-debugging/SKILL.md")).unwrap(),
        current
    );
    assert_eq!(
        std::fs::read(installed.join("root-cause-debugging/user-note.md")).unwrap(),
        b"keep"
    );
}

#[test]
fn preserves_a_personalized_hk_debug_bundle() {
    let temp = tempfile::tempdir().unwrap();
    let resources = temp.path().join("resources");
    let installed = temp.path().join("installed");
    let resource_bundle = resources.join("root-cause-debugging");
    let installed_bundle = installed.join("hk-debug");
    std::fs::create_dir_all(&resource_bundle).unwrap();
    std::fs::create_dir_all(&installed_bundle).unwrap();
    std::fs::write(resource_bundle.join("SKILL.md"), b"current").unwrap();
    std::fs::write(installed_bundle.join("SKILL.md"), b"personalized").unwrap();

    migrate_matching_legacy_hk_debug(&resources, &installed, &sha256_hex(b"official hk-debug"))
        .unwrap();

    assert_eq!(
        std::fs::read(installed.join("hk-debug/SKILL.md")).unwrap(),
        b"personalized"
    );
    assert!(!installed.join("root-cause-debugging").exists());
}
