use std::sync::Arc;

#[test]
fn reuses_one_lock_for_the_same_repository() {
    let repo = tempfile::tempdir().expect("repo");

    let first = super::subagent_git_lock::for_repo(repo.path()).expect("first lock");
    let second = super::subagent_git_lock::for_repo(repo.path()).expect("second lock");

    assert!(Arc::ptr_eq(&first, &second));
}

#[test]
fn keeps_different_repositories_independent() {
    let first_repo = tempfile::tempdir().expect("first repo");
    let second_repo = tempfile::tempdir().expect("second repo");

    let first = super::subagent_git_lock::for_repo(first_repo.path()).expect("first lock");
    let second = super::subagent_git_lock::for_repo(second_repo.path()).expect("second lock");

    assert!(!Arc::ptr_eq(&first, &second));
}
