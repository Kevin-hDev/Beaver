use super::{
    canonical_access_path, configured_roots_from_paths, decision_in_roots,
    ensure_allowed_in_roots, is_path_in_roots, normalize_allowed_paths,
};

#[test]
fn allows_exact_root_and_children_but_rejects_parent_and_sibling() {
    let temp = tempfile::tempdir().expect("temp");
    let allowed = temp.path().join("allowed");
    let child = allowed.join("child");
    let sibling = temp.path().join("sibling");
    std::fs::create_dir_all(&child).expect("child");
    std::fs::create_dir_all(&sibling).expect("sibling");
    let roots = vec![allowed.canonicalize().expect("allowed")];

    assert!(is_path_in_roots(&allowed.canonicalize().expect("root"), &roots));
    assert!(is_path_in_roots(&child.canonicalize().expect("child"), &roots));
    assert!(!is_path_in_roots(&temp.path().canonicalize().expect("parent"), &roots));
    assert!(!is_path_in_roots(&sibling.canonicalize().expect("sibling"), &roots));
}

#[test]
fn every_configured_root_grants_the_same_access() {
    let temp = tempfile::tempdir().expect("temp");
    let first = temp.path().join("first");
    let second = temp.path().join("second");
    let outside = temp.path().join("outside");
    for path in [&first, &second, &outside] {
        std::fs::create_dir_all(path.join("child")).expect("directory");
    }
    let roots = vec![
        dunce::canonicalize(&first).expect("first"),
        dunce::canonicalize(&second).expect("second"),
    ];

    for allowed in [&first, &second] {
        assert!(ensure_allowed_in_roots(&allowed.join("child"), &roots).is_ok());
    }
    assert!(ensure_allowed_in_roots(&outside.join("child"), &roots).is_err());
}

#[test]
fn canonicalizes_missing_descendants_from_the_nearest_existing_parent() {
    let temp = tempfile::tempdir().expect("temp");
    let allowed = temp.path().join("allowed");
    std::fs::create_dir_all(&allowed).expect("allowed");

    let candidate = canonical_access_path(&allowed.join("new").join("nested"))
        .expect("candidate");

    assert_eq!(candidate, allowed.canonicalize().expect("root").join("new/nested"));
}

#[test]
fn normalizes_deduplicates_and_bounds_configured_roots() {
    let temp = tempfile::tempdir().expect("temp");
    let value = temp.path().to_string_lossy().to_string();

    let normalized = normalize_allowed_paths(vec![value.clone(), value]).expect("normalized");

    assert_eq!(normalized.len(), 1);
    assert!(normalize_allowed_paths(Vec::new()).is_err());
    let roots = (0..super::MAX_ALLOWED_PATHS)
        .map(|index| {
            let path = temp.path().join(format!("root-{index}"));
            std::fs::create_dir(&path).expect("root");
            path.to_string_lossy().to_string()
        })
        .collect::<Vec<_>>();
    assert!(normalize_allowed_paths(roots).is_ok());
    assert!(normalize_allowed_paths(vec![
        "/".to_string();
        super::MAX_ALLOWED_PATHS + 1
    ])
    .is_err());
}

#[test]
fn unavailable_configured_root_does_not_hide_a_valid_root() {
    let temp = tempfile::tempdir().expect("temp");
    let valid = temp.path().join("valid");
    std::fs::create_dir(&valid).expect("valid");
    let missing = temp.path().join("missing");

    let roots = configured_roots_from_paths(vec![
        missing.to_string_lossy().to_string(),
        valid.to_string_lossy().to_string(),
    ])
    .expect("one root remains");

    assert_eq!(roots, vec![dunce::canonicalize(valid).expect("canonical")]);
    assert!(configured_roots_from_paths(vec![missing.to_string_lossy().to_string()]).is_err());
}

#[test]
fn decision_and_enforcement_share_the_same_canonical_policy() {
    let temp = tempfile::tempdir().expect("temp");
    let allowed = temp.path().join("allowed");
    let outside = temp.path().join("outside");
    std::fs::create_dir_all(allowed.join("child")).expect("allowed");
    std::fs::create_dir_all(&outside).expect("outside");
    let roots = vec![dunce::canonicalize(&allowed).expect("root")];

    let accepted = decision_in_roots(&allowed.join("child"), &roots).expect("decision");
    let rejected = decision_in_roots(&outside, &roots).expect("decision");

    assert!(accepted.allowed);
    assert_eq!(accepted.allowed_paths, vec![roots[0].to_string_lossy()]);
    assert!(!rejected.allowed);
    assert!(ensure_allowed_in_roots(&allowed.join("child"), &roots).is_ok());
    assert!(ensure_allowed_in_roots(&outside, &roots).is_err());
}

#[cfg(unix)]
#[test]
fn symlink_is_checked_against_its_real_target() {
    use std::os::unix::fs::symlink;

    let temp = tempfile::tempdir().expect("temp");
    let allowed = temp.path().join("allowed");
    let outside = temp.path().join("outside");
    std::fs::create_dir_all(&allowed).expect("allowed");
    std::fs::create_dir_all(&outside).expect("outside");
    symlink(&outside, allowed.join("escape")).expect("symlink");

    let candidate = canonical_access_path(&allowed.join("escape")).expect("candidate");
    let roots = vec![allowed.canonicalize().expect("allowed")];

    assert!(!is_path_in_roots(&candidate, &roots));
}
