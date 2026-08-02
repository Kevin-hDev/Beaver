use super::{
    canonical_access_path, is_path_in_roots, normalize_allowed_paths, roots_allow_shell,
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
    assert!(normalize_allowed_paths(vec!["/".to_string(); 33]).is_err());
}

#[test]
fn arbitrary_shell_requires_a_filesystem_root() {
    let temp = tempfile::tempdir().expect("temp");

    assert!(!roots_allow_shell(&[temp.path().to_path_buf()]));
    #[cfg(not(windows))]
    assert!(roots_allow_shell(&[std::path::PathBuf::from("/")]));
    #[cfg(windows)]
    assert!(roots_allow_shell(&[std::path::PathBuf::from("C:\\")]));
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
