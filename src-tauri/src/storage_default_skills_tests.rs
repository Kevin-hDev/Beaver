use super::{
    sha256_hex, sync_default_skills_from, ManagedSkillUpgrade, LEGACY_SKILL_CREATE_SHA256,
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
