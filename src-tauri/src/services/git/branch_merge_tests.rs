use super::{branch, branch_merge};
use git2::{Repository, Signature};
use std::path::Path;

#[test]
fn preview_counts_commits_and_reports_the_current_target() {
    let repo = init_repo();
    branch::create_branch(repo.path(), "feature").expect("create feature");
    commit_file(repo.path(), "feature.txt", "feature", "feature commit");
    branch::checkout_branch(repo.path(), "main").expect("checkout main");

    let preview = branch_merge::preview(repo.path(), "feature", "main").expect("preview");

    assert_eq!(preview.commits, 1);
    assert_eq!(preview.source_branch, "feature");
    assert_eq!(preview.target_branch, "main");
    assert!(preview.dirty_files.is_empty());
}

#[test]
fn detached_head_is_never_used_as_a_merge_target() {
    let repo = init_repo();
    branch::create_branch(repo.path(), "feature").expect("create feature");
    let opened = Repository::open(repo.path()).expect("open repo");
    let head = opened.head().unwrap().target().unwrap();
    opened.set_head_detached(head).expect("detach head");

    let error = branch_merge::preview(repo.path(), "main", "feature")
        .expect_err("detached target must fail closed");

    assert_eq!(error, branch_merge::MergeError::ContextChanged);
}

#[test]
fn existing_git_operation_is_preserved_and_blocks_merge() {
    let repo = init_repo();
    branch::create_branch(repo.path(), "feature").expect("create feature");
    commit_file(repo.path(), "feature.txt", "feature", "feature commit");
    branch::checkout_branch(repo.path(), "main").expect("checkout main");
    let opened = Repository::open(repo.path()).expect("open repo");
    let marker = opened.path().join("MERGE_HEAD");
    std::fs::write(
        &marker,
        opened.head().unwrap().target().unwrap().to_string(),
    )
    .expect("write merge state");

    let error = branch_merge::preview(repo.path(), "feature", "main")
        .expect_err("existing operation must block merge");

    assert_eq!(error, branch_merge::MergeError::InternalError);
    assert!(marker.exists());
}

#[test]
fn merge_adds_the_source_commits_and_keeps_both_branches() {
    let repo = init_repo();
    branch::create_branch(repo.path(), "feature").expect("create feature");
    commit_file(repo.path(), "feature.txt", "merged", "feature commit");
    branch::checkout_branch(repo.path(), "main").expect("checkout main");

    branch_merge::merge_current(repo.path(), "feature", "main", false, None)
        .expect("merge feature");

    assert_eq!(branch::get_context(repo.path()).branch, "main");
    assert!(
        branch_merge::preview(repo.path(), "feature", "main")
            .expect("merged preview")
            .commits
            == 0
    );
    assert_eq!(
        std::fs::read_to_string(repo.path().join("feature.txt")).unwrap(),
        "merged"
    );
}

#[test]
fn dirty_worktree_requires_an_explicit_commit() {
    let repo = init_repo();
    branch::create_branch(repo.path(), "feature").expect("create feature");
    commit_file(repo.path(), "feature.txt", "feature", "feature commit");
    branch::checkout_branch(repo.path(), "main").expect("checkout main");
    std::fs::write(repo.path().join("dirty.txt"), "keep me").expect("dirty file");

    let error = branch_merge::merge_current(repo.path(), "feature", "main", false, None)
        .expect_err("dirty merge must fail closed");

    assert_eq!(error, branch_merge::MergeError::DirtyWorktree);
    assert_eq!(
        std::fs::read_to_string(repo.path().join("dirty.txt")).unwrap(),
        "keep me"
    );
}

#[test]
fn commit_then_merge_preserves_local_changes() {
    let repo = init_repo();
    branch::create_branch(repo.path(), "feature").expect("create feature");
    commit_file(repo.path(), "feature.txt", "feature", "feature commit");
    branch::checkout_branch(repo.path(), "main").expect("checkout main");
    std::fs::write(repo.path().join("dirty.txt"), "preserved").expect("dirty file");

    branch_merge::merge_current(
        repo.path(),
        "feature",
        "main",
        true,
        Some("Local work".to_string()),
    )
    .expect("commit and merge");

    assert_eq!(
        std::fs::read_to_string(repo.path().join("dirty.txt")).unwrap(),
        "preserved"
    );
    assert_eq!(
        std::fs::read_to_string(repo.path().join("feature.txt")).unwrap(),
        "feature"
    );
}

#[test]
fn conflict_is_aborted_and_restores_the_target() {
    let repo = init_repo();
    branch::create_branch(repo.path(), "feature").expect("create feature");
    commit_file(repo.path(), "shared.txt", "feature", "feature change");
    branch::checkout_branch(repo.path(), "main").expect("checkout main");
    commit_file(repo.path(), "shared.txt", "main", "main change");

    let error = branch_merge::merge_current(repo.path(), "feature", "main", false, None)
        .expect_err("conflicting merge");

    assert_eq!(error, branch_merge::MergeError::MergeConflict);
    assert_eq!(branch::get_context(repo.path()).branch, "main");
    assert_eq!(
        std::fs::read_to_string(repo.path().join("shared.txt")).unwrap(),
        "main"
    );
    let opened = Repository::open(repo.path()).expect("open repo");
    assert!(!opened.path().join("MERGE_HEAD").exists());
}

#[cfg(unix)]
#[test]
fn repository_hooks_are_disabled_during_merge() {
    use std::os::unix::fs::PermissionsExt;

    let repo = init_repo();
    branch::create_branch(repo.path(), "feature").expect("create feature");
    commit_file(repo.path(), "feature.txt", "feature", "feature commit");
    branch::checkout_branch(repo.path(), "main").expect("checkout main");
    let marker = repo.path().join("hook-ran");
    let opened = Repository::open(repo.path()).expect("open repo");
    let hook = opened.path().join("hooks/post-merge");
    std::fs::write(&hook, format!("#!/bin/sh\ntouch '{}'\n", marker.display()))
        .expect("write hook");
    let mut permissions = std::fs::metadata(&hook)
        .expect("hook metadata")
        .permissions();
    permissions.set_mode(0o700);
    std::fs::set_permissions(&hook, permissions).expect("hook permissions");

    branch_merge::merge_current(repo.path(), "feature", "main", false, None)
        .expect("merge feature");

    assert!(!marker.exists());
}

fn init_repo() -> tempfile::TempDir {
    let tmp = tempfile::tempdir().expect("temp repo");
    let repo = Repository::init(tmp.path()).expect("init repo");
    let mut config = repo.config().expect("config");
    config.set_str("user.name", "CL-GO Test").expect("name");
    config
        .set_str("user.email", "test@example.com")
        .expect("email");
    std::fs::write(tmp.path().join("README.md"), "init").expect("initial file");
    commit_path(&repo, Path::new("README.md"), "init");
    repo.branch(
        "main",
        &repo.head().unwrap().peel_to_commit().unwrap(),
        false,
    )
    .expect("main branch");
    repo.set_head("refs/heads/main").expect("set main");
    drop(repo);
    tmp
}

fn commit_file(root: &Path, relative: &str, content: &str, message: &str) {
    std::fs::write(root.join(relative), content).expect("write file");
    let repo = Repository::open(root).expect("open repo");
    commit_path(&repo, Path::new(relative), message);
}

fn commit_path(repo: &Repository, path: &Path, message: &str) {
    let mut index = repo.index().expect("index");
    index.add_path(path).expect("add path");
    index.write().expect("write index");
    let tree_oid = index.write_tree().expect("tree oid");
    let tree = repo.find_tree(tree_oid).expect("tree");
    let signature = Signature::now("CL-GO Test", "test@example.com").expect("signature");
    let parent = repo.head().ok().and_then(|head| head.peel_to_commit().ok());
    let parents = parent.iter().collect::<Vec<_>>();
    repo.commit(
        Some("HEAD"),
        &signature,
        &signature,
        message,
        &tree,
        &parents,
    )
    .expect("commit");
}
