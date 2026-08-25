#[test]
fn manifest_budget_has_one_shared_authority() {
    let import_limits = include_str!("agent_import/limits.rs");
    let local_limits = include_str!("agent_local/skill_limits.rs");
    let default_skills = include_str!("../storage_default_skills.rs");
    let shared_path = "crate::services::skill_manifest_policy::MAX_SKILL_MANIFEST_BYTES";

    assert!(import_limits.contains(shared_path));
    assert!(!import_limits.contains("pub const MAX_MANIFEST_BYTES:"));
    assert!(local_limits.contains(shared_path));
    assert!(!local_limits.contains("pub const MAX_SKILL_CONTENT_BYTES:"));
    assert!(default_skills.contains(shared_path));
    assert!(!default_skills.contains("const MAX_SKILL_MANIFEST_BYTES:"));
}
