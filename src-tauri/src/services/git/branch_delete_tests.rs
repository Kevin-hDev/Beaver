use super::{action_error::GitActionError, branch, branch_delete};
use git2::{Repository, Signature};
use std::path::Path;

#[test]
fn preview_reports_dirty_files_and_unmerged_commits() {
    let tmp = init_repo();
    branch::create_branch(tmp.path(), "abandoned").expect("create branch");
    commit_file(tmp.path(), "feature.txt", "feature");
    std::fs::write(tmp.path().join("dirty.txt"), "dirty").expect("dirty file");

    let preview = branch_delete::preview(tmp.path(), "abandoned").expect("preview");

    assert!(preview.is_current);
    assert_eq!(preview.fallback_branch.as_deref(), Some("main"));
    assert_eq!(preview.unmerged_commits, 1);
    assert!(preview
        .dirty_files
        .iter()
        .any(|file| file.path == "dirty.txt"));
}

#[test]
fn preserve_merges_work_then_deletes_branch() {
    let tmp = init_repo();
    branch::create_branch(tmp.path(), "abandoned").expect("create branch");
    std::fs::write(tmp.path().join("feature.txt"), "preserved").expect("dirty file");

    branch_delete::preserve_and_delete(
        tmp.path(),
        "abandoned",
        Some("Travail conserve".to_string()),
    )
    .expect("preserve and delete");

    assert_eq!(branch::get_context(tmp.path()).branch, "main");
    assert!(!branch_delete::branch_exists(tmp.path(), "abandoned").expect("exists"));
    assert_eq!(
        std::fs::read_to_string(tmp.path().join("feature.txt")).expect("merged file"),
        "preserved",
    );
}

#[test]
fn discard_deletes_current_branch_and_dirty_files() {
    let tmp = init_repo();
    branch::create_branch(tmp.path(), "abandoned").expect("create branch");
    std::fs::write(tmp.path().join("dirty.txt"), "discard").expect("dirty file");

    branch_delete::discard_and_delete(tmp.path(), "abandoned").expect("discard and delete");

    assert_eq!(branch::get_context(tmp.path()).branch, "main");
    assert!(!tmp.path().join("dirty.txt").exists());
    assert!(!branch_delete::branch_exists(tmp.path(), "abandoned").expect("exists"));
}

#[test]
fn clean_delete_rechecks_unmerged_commits() {
    let tmp = init_repo();
    branch::create_branch(tmp.path(), "abandoned").expect("create branch");
    commit_file(tmp.path(), "feature.txt", "feature");
    branch::checkout_branch(tmp.path(), "main").expect("checkout main");

    assert_eq!(
        branch_delete::delete_clean(tmp.path(), "abandoned"),
        Err(GitActionError::UnmergedCommits { count: 1 }),
    );
    assert!(branch_delete::branch_exists(tmp.path(), "abandoned").expect("exists"));
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

fn commit_file(root: &Path, relative: &str, content: &str) {
    std::fs::write(root.join(relative), content).expect("write file");
    let repo = Repository::open(root).expect("open repo");
    commit_path(&repo, Path::new(relative), "feature");
}

fn commit_path(repo: &Repository, path: &Path, message: &str) {
    let mut index = repo.index().expect("index");
    index.add_path(path).expect("add path");
    index.write().expect("write index");
    let tree_oid = index.write_tree().expect("tree oid");
    let tree = repo.find_tree(tree_oid).expect("tree");
    let signature = Signature::now("CL-GO Test", "test@example.com").expect("signature");
    let parents = repo.head().ok().and_then(|head| head.peel_to_commit().ok());
    let parent_refs = parents.iter().collect::<Vec<_>>();
    repo.commit(
        Some("HEAD"),
        &signature,
        &signature,
        message,
        &tree,
        &parent_refs,
    )
    .expect("commit");
}
