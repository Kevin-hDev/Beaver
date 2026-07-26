use super::branch::{self, validate_branch_name, CreateBranchError};

fn init_repo_with_commit() -> tempfile::TempDir {
    let tmp = tempfile::tempdir().expect("temp repo");
    let repo = git2::Repository::init(tmp.path()).expect("init repo");
    std::fs::write(tmp.path().join("file.txt"), "initial").expect("write file");
    let mut index = repo.index().expect("index");
    index
        .add_path(std::path::Path::new("file.txt"))
        .expect("add");
    index.write().expect("write index");
    let tree_oid = index.write_tree().expect("tree");
    let tree = repo.find_tree(tree_oid).expect("find tree");
    let sig = git2::Signature::now("CL-GO Test", "test@example.com").expect("signature");
    repo.commit(Some("HEAD"), &sig, &sig, "init", &tree, &[])
        .expect("commit");
    drop(tree);
    drop(repo);
    tmp
}

#[test]
fn test_create_branch_on_unborn_repo_returns_clear_error() {
    let tmp = tempfile::tempdir().expect("temp repo");
    git2::Repository::init(tmp.path()).expect("init repo");

    let result = branch::create_branch(tmp.path(), "feature");

    assert_eq!(result, Err(CreateBranchError::UnbornHead));
    let msg = result.unwrap_err().to_string();
    assert!(
        msg.contains("commit") || msg.to_lowercase().contains("unborn"),
        "expected clear unborn-head message, got: {msg}"
    );
}

#[test]
fn test_create_branch_returns_already_exists_error() {
    let tmp = init_repo_with_commit();
    branch::create_branch(tmp.path(), "foo").expect("create branch");

    let result = branch::create_branch(tmp.path(), "foo");

    assert_eq!(result, Err(CreateBranchError::AlreadyExists));
}

#[test]
fn test_create_branch_returns_name_too_long_error() {
    let tmp = tempfile::tempdir().expect("temp repo");
    let name = "a".repeat(101);

    let result = branch::create_branch(tmp.path(), &name);

    assert_eq!(result, Err(CreateBranchError::NameTooLong));
}

#[test]
fn test_validate_branch_name_is_pub() {
    validate_branch_name("feature/foo").expect("public validator should be callable");
}
