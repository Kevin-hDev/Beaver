use super::{blob_preview, commit_files, history};
use git2::{Oid, Repository, Signature};
use std::path::Path;

#[test]
fn paginates_current_branch_history() {
    let fixture = fixture();
    let first = history::list_commits(fixture.dir.path(), &fixture.branch, None, Some(2))
        .expect("first page");
    assert_eq!(first.commits.len(), 2);
    assert_eq!(first.commits[0].message, "fourth");
    assert!(first.next_cursor.is_some());

    let second = history::list_commits(
        fixture.dir.path(),
        &fixture.branch,
        first.next_cursor.as_deref(),
        Some(2),
    )
    .expect("second page");
    assert_eq!(second.commits.len(), 2);
    assert_eq!(second.commits[1].message, "first");
    assert!(second.next_cursor.is_none());
}

#[test]
fn lists_deleted_file_and_reads_parent_version() {
    let fixture = fixture();
    let files = commit_files::list_commit_files(
        fixture.dir.path(),
        &fixture.branch,
        &fixture.deletion.to_string(),
    )
    .expect("commit files");
    assert!(files
        .iter()
        .any(|file| file.path == "deleted.txt" && file.status == "deleted"));

    let bytes = blob_preview::read_blob_with_limit(
        fixture.dir.path(),
        &fixture.branch,
        &fixture.deletion.to_string(),
        "deleted.txt",
        true,
        blob_preview::MAX_GIT_BLOB_SIZE,
    )
    .expect("parent blob");
    assert_eq!(bytes, b"before deletion\n");
}

#[test]
fn rejects_traversal_and_reports_bounded_uncommitted_files() {
    let fixture = fixture();
    assert!(blob_preview::read_blob_with_limit(
        fixture.dir.path(),
        &fixture.branch,
        &fixture.deletion.to_string(),
        "../secret.txt",
        false,
        blob_preview::MAX_GIT_BLOB_SIZE,
    )
    .is_err());

    std::fs::write(fixture.dir.path().join("working.txt"), "pending\n").expect("working file");
    let snapshot =
        history::list_uncommitted(fixture.dir.path(), &fixture.branch).expect("uncommitted");
    assert!(snapshot.files.len() <= 200);
    assert_eq!(snapshot.total_files, 1);
    assert!(!snapshot.truncated);
    assert!(snapshot.files.iter().any(|file| file.path == "working.txt"));
}

#[test]
fn reports_when_the_uncommitted_file_list_is_truncated() {
    let fixture = fixture();
    for index in 0..201 {
        std::fs::write(
            fixture.dir.path().join(format!("pending-{index}.txt")),
            "pending\n",
        )
        .expect("working file");
    }

    let snapshot =
        history::list_uncommitted(fixture.dir.path(), &fixture.branch).expect("uncommitted");
    assert_eq!(snapshot.files.len(), 200);
    assert_eq!(snapshot.total_files, 201);
    assert!(snapshot.truncated);
}

#[test]
fn rejects_a_blob_before_copying_it_past_the_preview_limit() {
    let fixture = fixture();
    let result = blob_preview::read_blob_with_limit(
        fixture.dir.path(),
        &fixture.branch,
        &fixture.deletion.to_string(),
        "deleted.txt",
        true,
        4,
    );

    assert!(result.is_err());
    assert_eq!(blob_preview::MAX_GIT_BINARY_PREVIEW_SIZE, 10 * 1024 * 1024);
}

#[test]
fn bounds_long_utf8_commit_messages() {
    let fixture = fixture();
    let repo = Repository::open(fixture.dir.path()).expect("repo");
    let message = "é".repeat(1_000);
    commit(&repo, "long.txt", Some("long\n"), &message);

    let page =
        history::list_commits(fixture.dir.path(), &fixture.branch, None, Some(1)).expect("history");
    assert!(page.commits[0].message.chars().count() <= 160);
}

struct Fixture {
    dir: tempfile::TempDir,
    branch: String,
    deletion: Oid,
}

fn fixture() -> Fixture {
    let dir = tempfile::tempdir().expect("repo");
    let repo = Repository::init(dir.path()).expect("init");
    commit(&repo, "a.txt", Some("one\n"), "first");
    commit(&repo, "deleted.txt", Some("before deletion\n"), "second");
    let deletion = commit(&repo, "deleted.txt", None, "third");
    commit(&repo, "b.txt", Some("four\n"), "fourth");
    let branch = repo
        .head()
        .expect("head")
        .shorthand()
        .expect("branch")
        .to_string();
    Fixture {
        dir,
        branch,
        deletion,
    }
}

fn commit(repo: &Repository, path: &str, content: Option<&str>, message: &str) -> Oid {
    let workdir = repo.workdir().expect("workdir");
    let mut index = repo.index().expect("index");
    if let Some(value) = content {
        std::fs::write(workdir.join(path), value).expect("write");
        index.add_path(Path::new(path)).expect("add");
    } else {
        std::fs::remove_file(workdir.join(path)).expect("remove");
        index.remove_path(Path::new(path)).expect("remove index");
    }
    index.write().expect("index write");
    let tree_oid = index.write_tree().expect("tree oid");
    let tree = repo.find_tree(tree_oid).expect("tree");
    let signature = Signature::now("CL-GO Test", "test@example.com").expect("signature");
    let parent = repo.head().ok().and_then(|head| head.peel_to_commit().ok());
    let parents: Vec<&git2::Commit<'_>> = parent.iter().collect();
    repo.commit(
        Some("HEAD"),
        &signature,
        &signature,
        message,
        &tree,
        &parents,
    )
    .expect("commit")
}
