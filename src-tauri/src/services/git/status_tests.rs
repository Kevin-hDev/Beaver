use super::status::list_dirty_files;
use git2::{Repository, Signature};
use std::path::Path;

#[test]
fn reports_modified_and_untracked_line_stats() {
    let fixture = fixture();
    std::fs::write(fixture.path().join("tracked.txt"), "new\nextra\n").expect("modify");
    std::fs::write(fixture.path().join("new.txt"), "one\ntwo\n").expect("untracked");

    let files = list_dirty_files(fixture.path()).expect("dirty files");
    let modified = files
        .iter()
        .find(|file| file.path == "tracked.txt")
        .expect("modified");
    let added = files
        .iter()
        .find(|file| file.path == "new.txt")
        .expect("new");
    assert_eq!((modified.additions, modified.deletions), (2, 1));
    assert_eq!((added.additions, added.deletions), (2, 0));
}

#[test]
fn reports_staged_changes_against_head() {
    let fixture = fixture();
    std::fs::write(fixture.path().join("tracked.txt"), "staged\nextra\n").expect("modify");
    let repo = Repository::open(fixture.path()).expect("repo");
    let mut index = repo.index().expect("index");
    index.add_path(Path::new("tracked.txt")).expect("stage");
    index.write().expect("index write");

    let files = list_dirty_files(fixture.path()).expect("dirty files");
    let file = files
        .iter()
        .find(|file| file.path == "tracked.txt")
        .expect("staged");
    assert_eq!((file.additions, file.deletions), (2, 1));
}

fn fixture() -> tempfile::TempDir {
    let dir = tempfile::tempdir().expect("repo");
    let repo = Repository::init(dir.path()).expect("init");
    std::fs::write(dir.path().join("tracked.txt"), "old\n").expect("file");
    let mut index = repo.index().expect("index");
    index.add_path(Path::new("tracked.txt")).expect("add");
    index.write().expect("index write");
    let tree_id = index.write_tree().expect("tree id");
    let tree = repo.find_tree(tree_id).expect("tree");
    let signature = Signature::now("CL-GO Test", "test@example.com").expect("signature");
    repo.commit(Some("HEAD"), &signature, &signature, "init", &tree, &[])
        .expect("commit");
    drop(tree);
    drop(repo);
    dir
}
