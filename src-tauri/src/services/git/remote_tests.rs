use super::{branch, remote};
use git2::{Error, ErrorClass, ErrorCode, Repository, Signature};
use std::path::Path;

#[test]
fn publishes_branch_and_tracks_ahead_commits() {
    let local = init_repo();
    let bare = tempfile::tempdir().expect("bare dir");
    Repository::init_bare(bare.path()).expect("bare repo");
    let repo = Repository::open(local.path()).expect("open local");
    repo.remote("origin", bare.path().to_str().expect("remote path"))
        .expect("remote");
    drop(repo);

    let initial = remote::status(local.path()).expect("initial status");
    assert!(initial.has_remote);
    assert!(!initial.has_upstream);

    remote::push_current(local.path(), None, None).expect("first push");
    let published = remote::status(local.path()).expect("published status");
    assert!(published.has_upstream);
    assert!(published.has_remote_branch);
    assert_eq!(published.ahead, 0);

    std::fs::write(local.path().join("next.txt"), "next").expect("next file");
    commit_path(local.path(), "next.txt", "next");
    assert_eq!(remote::status(local.path()).expect("ahead status").ahead, 1);

    remote::push_current(local.path(), None, None).expect("second push");
    assert_eq!(
        remote::status(local.path()).expect("synced status").ahead,
        0
    );
    assert_eq!(branch::get_context(local.path()).branch, "main");
}

#[test]
fn remote_branch_without_upstream_is_detected_and_counts_ahead_commits() {
    let local = init_repo();
    let bare = tempfile::tempdir().expect("bare dir");
    Repository::init_bare(bare.path()).expect("bare repo");
    let repo = Repository::open(local.path()).expect("open local");
    repo.remote("origin", bare.path().to_str().expect("remote path"))
        .expect("remote");
    drop(repo);

    remote::push_current(local.path(), None, None).expect("seed remote branch");
    let repo = Repository::open(local.path()).expect("reopen local");
    repo.find_branch("main", git2::BranchType::Local)
        .expect("main branch")
        .set_upstream(None)
        .expect("remove upstream");
    drop(repo);

    std::fs::write(local.path().join("ahead.txt"), "ahead").expect("ahead file");
    commit_path(local.path(), "ahead.txt", "ahead");

    let status = remote::status(local.path()).expect("remote status");
    assert!(!status.has_upstream);
    assert!(status.has_remote_branch);
    assert_eq!(status.ahead, 1);
    assert_eq!(status.behind, 0);
}

#[test]
fn github_ssh_remote_does_not_require_oauth_token() {
    let local = init_repo();
    let repo = Repository::open(local.path()).expect("open local");
    repo.remote("origin", "git@github.com:example/project.git")
        .expect("ssh remote");
    drop(repo);

    assert!(!remote::remote_requires_github_token(local.path()));
    assert!(remote::status(local.path()).expect("ssh status").is_github);
    let repo = Repository::open(local.path()).expect("reopen local");
    repo.remote_set_url("origin", "https://github.com/example/project.git")
        .expect("https remote");
    drop(repo);
    assert!(remote::remote_requires_github_token(local.path()));
}

#[test]
fn push_errors_are_classified_without_exposing_internal_details() {
    let auth = Error::new(ErrorCode::Auth, ErrorClass::Http, "private auth details");
    assert_eq!(
        remote::map_push_error(auth),
        remote::PushError::AuthenticationRequired
    );

    let forbidden = Error::new(ErrorCode::GenericError, ErrorClass::Http, "HTTP 403");
    assert_eq!(
        remote::map_push_error(forbidden),
        remote::PushError::PermissionDenied
    );

    let network = Error::new(ErrorCode::Timeout, ErrorClass::Net, "private host");
    assert_eq!(
        remote::map_push_error(network),
        remote::PushError::NetworkUnavailable
    );
}

#[test]
fn push_refuses_when_the_selected_branch_changed() {
    let local = init_repo();
    let bare = tempfile::tempdir().expect("bare dir");
    Repository::init_bare(bare.path()).expect("bare repo");
    let repo = Repository::open(local.path()).expect("open local");
    repo.remote("origin", bare.path().to_str().expect("remote path"))
        .expect("remote");
    drop(repo);

    let error = remote::push_current(local.path(), Some("other"), None)
        .expect_err("changed branch must block push");

    assert_eq!(error, remote::PushError::ContextChanged);
    assert!(
        !remote::status(local.path())
            .expect("status")
            .has_remote_branch
    );
}

fn init_repo() -> tempfile::TempDir {
    let tmp = tempfile::tempdir().expect("temp repo");
    let repo = Repository::init(tmp.path()).expect("init repo");
    let mut config = repo.config().expect("config");
    config.set_str("user.name", "Beaver Test").expect("name");
    config
        .set_str("user.email", "test@example.com")
        .expect("email");
    std::fs::write(tmp.path().join("README.md"), "init").expect("readme");
    commit_path(tmp.path(), "README.md", "init");
    let repo = Repository::open(tmp.path()).expect("reopen repo");
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

fn commit_path(root: &Path, relative: &str, message: &str) {
    let repo = Repository::open(root).expect("open repo");
    let mut index = repo.index().expect("index");
    index.add_path(Path::new(relative)).expect("add path");
    index.write().expect("write index");
    let tree_oid = index.write_tree().expect("tree oid");
    let tree = repo.find_tree(tree_oid).expect("tree");
    let signature = Signature::now("Beaver Test", "test@example.com").expect("signature");
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
