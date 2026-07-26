use super::{branch, branch_commit};
use git2::{Repository, Signature};
use std::path::Path;

#[test]
fn commits_changes_without_switching_branch() {
    let tmp = init_repo();
    let initial_branch = current_branch_name(tmp.path());
    std::fs::write(tmp.path().join("keep.txt"), "changed").expect("change file");

    branch_commit::commit_all(tmp.path(), Some("Résumé utilisateur".to_string()))
        .expect("commit changes");

    assert_eq!(current_branch_name(tmp.path()), initial_branch);
    assert_eq!(branch::get_context(tmp.path()).dirty_count, 0);
    let repo = Repository::open(tmp.path()).expect("open repo");
    assert_eq!(
        repo.head().unwrap().peel_to_commit().unwrap().message(),
        Ok("Résumé utilisateur"),
    );
}

fn current_branch_name(root: &Path) -> String {
    Repository::open(root)
        .expect("open repo")
        .head()
        .expect("head")
        .shorthand()
        .expect("valid branch name")
        .to_string()
}

fn init_repo() -> tempfile::TempDir {
    let tmp = tempfile::tempdir().expect("temp repo");
    let repo = Repository::init(tmp.path()).expect("init repo");
    let mut cfg = repo.config().expect("config");
    cfg.set_str("user.name", "CL-GO Test").expect("name");
    cfg.set_str("user.email", "test@example.com")
        .expect("email");
    std::fs::write(tmp.path().join("keep.txt"), "keep").expect("write file");
    let mut index = repo.index().expect("index");
    index.add_path(Path::new("keep.txt")).expect("add path");
    let tree_oid = index.write_tree().expect("tree");
    let tree = repo.find_tree(tree_oid).expect("find tree");
    let sig = Signature::now("CL-GO Test", "test@example.com").expect("signature");
    repo.commit(Some("HEAD"), &sig, &sig, "init", &tree, &[])
        .expect("commit");
    drop(tree);
    drop(repo);
    tmp
}
