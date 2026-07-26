use super::{branch, branch_commit};
use git2::{BranchType, Repository, Signature};
use std::path::Path;

#[test]
fn parses_git_author_ident() {
    let parsed = branch_commit::parse_git_ident("Kevin Huynh <kevin@example.com> 1779207754 +0200");
    assert_eq!(
        parsed,
        Some(("Kevin Huynh".to_string(), "kevin@example.com".to_string()))
    );
}

#[test]
fn commits_deletions_then_switches_branch() {
    let tmp = init_repo();
    create_branch(&tmp, "target");
    std::fs::remove_file(tmp.path().join("remove.txt")).expect("delete file");
    std::fs::write(tmp.path().join("new.txt"), "new").expect("new file");

    branch_commit::commit_all_and_checkout(tmp.path(), "target", None)
        .expect("commit and checkout");

    let ctx = branch::get_context(tmp.path());
    assert_eq!(ctx.branch, "target");
    assert_eq!(ctx.dirty_count, 0);
}

#[test]
fn default_commit_message_uses_beaver() {
    let tmp = init_repo();
    std::fs::write(tmp.path().join("keep.txt"), "changed").expect("change file");

    branch_commit::commit_all(tmp.path(), None).expect("commit changes");

    let repo = Repository::open(tmp.path()).expect("open repo");
    let message = repo
        .head()
        .expect("head")
        .peel_to_commit()
        .expect("commit")
        .message()
        .expect("message")
        .to_string();
    assert_eq!(message, "Save changes from Beaver");
}

#[test]
fn appends_optional_description_to_wip_commit() {
    let tmp = init_repo();
    create_branch(&tmp, "target");
    let source_branch = current_branch_name(tmp.path());
    std::fs::write(tmp.path().join("keep.txt"), "changed").expect("change file");

    branch_commit::commit_all_and_checkout(
        tmp.path(),
        "target",
        Some("Résumé utilisateur\navec détails".to_string()),
    )
    .expect("commit and checkout");

    let repo = Repository::open(tmp.path()).expect("open repo");
    let source_ref = repo
        .find_branch(&source_branch, BranchType::Local)
        .expect("source branch");
    let message = source_ref
        .get()
        .peel_to_commit()
        .unwrap()
        .message()
        .unwrap()
        .to_string();
    assert!(message.contains("WIP: save changes before switching to target"));
    assert!(message.contains("\n\nRésumé utilisateur\navec détails"));
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
    cfg.set_str("user.name", "Beaver Test").expect("name");
    cfg.set_str("user.email", "test@example.com")
        .expect("email");
    std::fs::write(tmp.path().join("keep.txt"), "keep").expect("keep");
    std::fs::write(tmp.path().join("remove.txt"), "remove").expect("remove");
    commit_paths(&repo, tmp.path(), &["keep.txt", "remove.txt"]);
    drop(repo);
    tmp
}

fn create_branch(tmp: &tempfile::TempDir, name: &str) {
    let repo = Repository::open(tmp.path()).expect("open repo");
    let head = repo.head().unwrap().peel_to_commit().unwrap();
    repo.branch(name, &head, false).expect("create branch");
}

fn commit_paths(repo: &Repository, root: &Path, paths: &[&str]) {
    let mut index = repo.index().expect("index");
    for path in paths {
        index.add_path(Path::new(path)).expect("add path");
    }
    index.write().expect("write index");
    let tree_oid = index.write_tree().expect("tree");
    let tree = repo.find_tree(tree_oid).expect("find tree");
    let sig = Signature::now("Beaver Test", "test@example.com").expect("signature");
    repo.commit(Some("HEAD"), &sig, &sig, "init", &tree, &[])
        .expect("commit");
    assert!(root.join("keep.txt").exists());
}
