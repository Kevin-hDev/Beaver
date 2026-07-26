use super::{diff_preview, status};
use git2::{IndexAddOption, Oid, Repository, Signature};

#[test]
fn previews_working_changes_for_all_text_statuses() {
    let fixture = fixture();
    change_files(&fixture.repo);
    let head = fixture.repo.head().unwrap().target().unwrap().to_string();

    let modified = working(&fixture, &head, "modified.txt", None);
    assert_line(&modified, "deleted", "old");
    assert_line(&modified, "added", "new");

    let added = working(&fixture, &head, "added.txt", None);
    assert_line(&added, "added", "created");

    let deleted = working(&fixture, &head, "deleted.txt", None);
    assert_line(&deleted, "deleted", "removed");

    let renamed = working(&fixture, &head, "new-name.txt", Some("old-name.txt"));
    assert_line(&renamed, "deleted", "one");
    assert_line(&renamed, "added", "ONE");

    let pure_rename = working(&fixture, &head, "pure-new.txt", Some("pure-old.txt"));
    assert!(!pure_rename.binary);
    assert!(pure_rename.hunks.is_empty());

    let dirty = status::list_dirty_files(fixture.repo.workdir().unwrap()).unwrap();
    assert!(dirty.iter().any(|file| {
        file.path == "new-name.txt"
            && file.previous_path.as_deref() == Some("old-name.txt")
            && file.status == "renamed"
    }));
}

#[test]
fn previews_commit_changes_and_rejects_invalid_context() {
    let fixture = fixture();
    change_files(&fixture.repo);
    let commit = commit_all(&fixture.repo, "changes");

    let modified = committed(&fixture, commit, "modified.txt", None);
    assert_line(&modified, "deleted", "old");
    assert_line(&modified, "added", "new");

    let added = committed(&fixture, commit, "added.txt", None);
    assert_line(&added, "added", "created");

    let deleted = committed(&fixture, commit, "deleted.txt", None);
    assert_line(&deleted, "deleted", "removed");

    let renamed = committed(&fixture, commit, "new-name.txt", Some("old-name.txt"));
    assert_line(&renamed, "deleted", "one");
    assert_line(&renamed, "added", "ONE");

    let pure_rename = committed(&fixture, commit, "pure-new.txt", Some("pure-old.txt"));
    assert!(!pure_rename.binary);
    assert!(pure_rename.hunks.is_empty());

    assert!(diff_preview::read_commit_diff(
        fixture.repo.workdir().unwrap(),
        &fixture.branch,
        &commit.to_string(),
        "../secret.txt",
        None,
    )
    .is_err());
    assert!(diff_preview::read_working_diff(
        fixture.repo.workdir().unwrap(),
        &fixture.branch,
        &"0".repeat(40),
        "modified.txt",
        None,
    )
    .is_err());
}

fn working(
    fixture: &Fixture,
    head: &str,
    path: &str,
    previous: Option<&str>,
) -> diff_preview::GitDiffPreview {
    diff_preview::read_working_diff(
        fixture.repo.workdir().unwrap(),
        &fixture.branch,
        head,
        path,
        previous,
    )
    .unwrap_or_else(|error| panic!("working diff for {path}: {error}"))
}

fn committed(
    fixture: &Fixture,
    commit: Oid,
    path: &str,
    previous: Option<&str>,
) -> diff_preview::GitDiffPreview {
    diff_preview::read_commit_diff(
        fixture.repo.workdir().unwrap(),
        &fixture.branch,
        &commit.to_string(),
        path,
        previous,
    )
    .expect("commit diff")
}

fn assert_line(preview: &diff_preview::GitDiffPreview, kind: &str, content: &str) {
    assert!(!preview.binary);
    assert!(preview
        .hunks
        .iter()
        .flat_map(|hunk| &hunk.lines)
        .any(|line| {
            line.kind == kind
                && line.content == content
                && (line.old_line.is_some() || line.new_line.is_some())
        }));
}

struct Fixture {
    repo: Repository,
    branch: String,
}

fn fixture() -> Fixture {
    let dir = tempfile::tempdir().expect("temp repo");
    let path = dir.keep();
    let repo = Repository::init(&path).expect("init repo");
    let workdir = repo.workdir().unwrap();
    std::fs::write(workdir.join("modified.txt"), "old\nsame\n").unwrap();
    std::fs::write(workdir.join("deleted.txt"), "removed\n").unwrap();
    std::fs::write(
        workdir.join("old-name.txt"),
        "one\ntwo\nthree\nfour\nfive\n",
    )
    .unwrap();
    std::fs::write(workdir.join("pure-old.txt"), "same content\n").unwrap();
    commit_all(&repo, "initial");
    let branch = repo.head().unwrap().shorthand().unwrap().to_string();
    Fixture { repo, branch }
}

fn change_files(repo: &Repository) {
    let workdir = repo.workdir().unwrap();
    std::fs::write(workdir.join("modified.txt"), "new\nsame\n").unwrap();
    std::fs::write(workdir.join("added.txt"), "created\n").unwrap();
    std::fs::remove_file(workdir.join("deleted.txt")).unwrap();
    std::fs::rename(workdir.join("old-name.txt"), workdir.join("new-name.txt")).unwrap();
    std::fs::rename(workdir.join("pure-old.txt"), workdir.join("pure-new.txt")).unwrap();
    std::fs::write(
        workdir.join("new-name.txt"),
        "ONE\ntwo\nthree\nfour\nfive\n",
    )
    .unwrap();
}

fn commit_all(repo: &Repository, message: &str) -> Oid {
    let mut index = repo.index().unwrap();
    index.add_all(["*"], IndexAddOption::DEFAULT, None).unwrap();
    index.write().unwrap();
    let tree_id = index.write_tree().unwrap();
    let tree = repo.find_tree(tree_id).unwrap();
    let signature = Signature::now("CL-GO Test", "test@example.com").unwrap();
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
    .unwrap()
}
