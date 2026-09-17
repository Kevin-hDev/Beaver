use super::*;
use git2::{Repository, Signature};

fn init_repo_with_commit() -> tempfile::TempDir {
    let tmp = tempfile::tempdir().expect("temp repo");
    let repo = Repository::init(tmp.path()).expect("init repo");
    std::fs::write(tmp.path().join("README.md"), "init").expect("write file");
    let mut index = repo.index().expect("index");
    index
        .add_path(std::path::Path::new("README.md"))
        .expect("add");
    index.write().expect("write index");
    let tree_id = index.write_tree().expect("tree");
    let tree = repo.find_tree(tree_id).expect("find tree");
    let sig = Signature::now("CL-GO Test", "test@example.com").expect("signature");
    repo.commit(Some("HEAD"), &sig, &sig, "init", &tree, &[])
        .expect("commit");
    tmp
}

#[test]
fn branch_name_shape_is_valid() {
    let name = random_branch_name();
    assert!(name.starts_with("clone-"));
    assert_eq!(name.len(), 14);
    assert!(branch::validate_branch_name(&name).is_ok());
}

#[test]
fn hex_lower_formats_bytes() {
    assert_eq!(hex_lower(&[0xab, 0x01, 0x9f, 0x00]), "ab019f00");
}

#[tokio::test]
async fn unique_branch_retries_on_collision() {
    let tmp = init_repo_with_commit();
    branch::create_branch(tmp.path(), "clone-00000000").expect("seed branch");
    let result = create_unique_branch_from_candidates(
        tmp.path().to_path_buf(),
        ["clone-00000000".to_string(), "clone-11111111".to_string()].to_vec(),
    )
    .await
    .expect("create branch");
    assert_eq!(result, "clone-11111111");
}

#[tokio::test]
async fn unique_branch_fails_after_three_collisions() {
    let tmp = init_repo_with_commit();
    for name in ["clone-00000000", "clone-11111111", "clone-22222222"] {
        branch::create_branch(tmp.path(), name).expect("seed branch");
    }
    let result = create_unique_branch_from_candidates(
        tmp.path().to_path_buf(),
        ["clone-00000000", "clone-11111111", "clone-22222222"]
            .map(str::to_string)
            .to_vec(),
    )
    .await;
    assert_eq!(result, Err(branch::CreateBranchError::AlreadyExists));
}
