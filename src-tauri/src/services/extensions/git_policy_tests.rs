use super::source_validation::GitSource;
use super::OperationFailure;
use std::time::Duration;

fn source(reference: Option<&str>) -> GitSource {
    GitSource {
        locator: "https://git.example/beaver/extension.git".to_string(),
        clone_url: "https://git.example/beaver/extension.git".to_string(),
        reference: reference.map(str::to_string),
    }
}

#[test]
fn ambiguous_short_hashes_start_shallow_but_full_commits_do_not() {
    let branch = source(Some("main"));
    let commit = source(Some("0123456789abcdef0123456789abcdef01234567"));
    let abbreviated_commit = source(Some("0123456"));

    assert!(super::git_source::should_use_shallow_clone(&branch));
    assert!(!super::git_source::should_use_shallow_clone(&commit));
    assert!(super::git_source::should_use_shallow_clone(
        &abbreviated_commit
    ));
}

#[test]
fn local_fixtures_never_request_a_shallow_transport() {
    let mut local = source(None);
    local.clone_url = "file:///tmp/beaver-extension.git".to_string();

    assert!(!super::git_source::should_use_shallow_clone(&local));
}

#[test]
fn elapsed_network_deadline_is_reported_as_a_timeout() {
    let error = git2::Error::new(
        git2::ErrorCode::GenericError,
        git2::ErrorClass::Net,
        "generic transport failure",
    );
    let threshold = crate::services::git::network_policy::timeout_classification_threshold();

    assert_eq!(
        super::git_source::clone_failure(&error, threshold),
        OperationFailure::GitTimeout
    );
    assert_eq!(
        super::git_source::clone_failure(&error, Duration::from_secs(1)),
        OperationFailure::GitDownloadFailed
    );
}
