use super::{action_error::GitActionError, branch};

fn init_repo_with_commit() -> tempfile::TempDir {
    let tmp = tempfile::tempdir().expect("temp repo");
    let repo = git2::Repository::init(tmp.path()).expect("init repo");
    std::fs::write(tmp.path().join("file.txt"), "initial").expect("write file");
    let mut index = repo.index().expect("index");
    index
        .add_path(std::path::Path::new("file.txt"))
        .expect("add file");
    index.write().expect("write index");
    let tree_id = index.write_tree().expect("write tree");
    let tree = repo.find_tree(tree_id).expect("find tree");
    let signature = git2::Signature::now("CL-GO Test", "test@example.com").expect("signature");
    repo.commit(Some("HEAD"), &signature, &signature, "init", &tree, &[])
        .expect("commit");
    drop(tree);
    drop(repo);
    tmp
}

#[test]
fn serializes_stable_error_kind_without_internal_details() {
    let error = GitActionError::DirtyWorktree { dirty_count: 3 };
    let value = serde_json::to_value(error).expect("serialize git error");

    assert_eq!(value["kind"], "dirty_worktree");
    assert_eq!(value["dirty_count"], 3);
    assert!(value.get("message").is_none());
}

#[test]
fn display_is_safe_and_does_not_contain_paths() {
    assert_eq!(
        GitActionError::RepositoryUnavailable.to_string(),
        "repository unavailable"
    );
}

#[test]
fn checkout_reports_dirty_file_count_as_structured_error() {
    let tmp = init_repo_with_commit();
    branch::create_branch(tmp.path(), "feature").expect("create feature");
    std::fs::write(tmp.path().join("file.txt"), "dirty").expect("dirty file");

    let error = branch::checkout_branch(tmp.path(), "master").expect_err("checkout must fail");

    assert_eq!(error, GitActionError::DirtyWorktree { dirty_count: 1 });
}

#[test]
fn checkout_reports_missing_branch_without_git_details() {
    let tmp = init_repo_with_commit();

    let error = branch::checkout_branch(tmp.path(), "missing").expect_err("checkout must fail");

    assert_eq!(error, GitActionError::BranchUnavailable);
}
