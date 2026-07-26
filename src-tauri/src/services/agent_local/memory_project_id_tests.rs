use super::*;

#[test]
fn project_folder_is_human_readable_and_collision_safe() {
    let root = tempfile::tempdir().unwrap();
    let project = root.path().join("Projects").join("CL-GO-DASH");
    std::fs::create_dir_all(&project).unwrap();

    let identity = project_identity(&project).unwrap();

    assert!(identity.id.contains("-Projects-CL-GO-DASH--"));
    assert!(identity.id.ends_with(&identity.legacy_id[..8]));
    assert!(valid_project_id(&identity.id));
}

#[test]
fn same_project_name_in_two_paths_keeps_distinct_ids() {
    let root = tempfile::tempdir().unwrap();
    let first = root.path().join("one").join("project");
    let second = root.path().join("two").join("project");
    std::fs::create_dir_all(&first).unwrap();
    std::fs::create_dir_all(&second).unwrap();

    assert_ne!(
        project_identity(&first).unwrap().id,
        project_identity(&second).unwrap().id
    );
}

#[test]
fn slug_replaces_platform_unsafe_characters() {
    assert_eq!(
        readable_slug("/Users/kevinh/Projects/CL GO/.claude"),
        "-Users-kevinh-Projects-CL-GO--claude"
    );
}

#[test]
fn long_paths_keep_the_useful_tail_within_the_limit() {
    let slug = format!("/Users/{}", "deep-project-".repeat(40));
    let truncated = truncate_slug(&readable_slug(&slug), 170);

    assert!(truncated.len() <= 170);
    assert!(truncated.ends_with("deep-project-"));
}

#[test]
fn validator_accepts_legacy_ids_during_migration_only() {
    assert!(valid_project_id("d316bc220f786a9b54eca7b0"));
    assert!(valid_project_id("-Users-kevinh-Projects-demo--0123abcd"));
    assert!(!valid_project_id("../demo--0123abcd"));
    assert!(!valid_project_id("demo--0123ABCD"));
    assert!(!valid_project_id("demo--0123abcg"));
    assert!(!valid_project_id("demo--0123abcd/other"));
}
