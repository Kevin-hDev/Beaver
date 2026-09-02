use super::loading_marker::{self, MarkerRead};
use std::collections::HashSet;

const ID: &str = "com.example.crash";

#[test]
fn marker_round_trip_tracks_attempts_and_host_stages() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("extension-loading.json");

    loading_marker::start_at(&path, ID, 1).unwrap();
    let MarkerRead::Valid(marker) = loading_marker::read_at(&path) else {
        panic!("valid marker expected");
    };
    assert_eq!(marker.extension_id, ID);
    assert_eq!(marker.stage, "import");
    assert_eq!(marker.attempts, 1);
    assert!(marker.can_retry());

    loading_marker::advance_at(&path, ID, "activate").unwrap();
    loading_marker::advance_at(&path, ID, "register").unwrap();
    let MarkerRead::Valid(marker) = loading_marker::read_at(&path) else {
        panic!("valid marker expected");
    };
    assert_eq!(marker.stage, "register");

    loading_marker::start_at(&path, ID, 3).unwrap();
    let MarkerRead::Valid(marker) = loading_marker::read_at(&path) else {
        panic!("valid marker expected");
    };
    assert!(!marker.can_retry());
}

#[test]
fn cautious_retry_is_only_counted_when_the_next_host_load_starts() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("extension-loading.json");
    loading_marker::start_at(&path, ID, 1).unwrap();

    assert_eq!(loading_marker::next_retry_attempt_at(&path, ID).unwrap(), 2);
    let MarkerRead::Valid(marker) = loading_marker::read_at(&path) else {
        panic!("retry marker expected");
    };
    assert_eq!(marker.attempts, 1);

    loading_marker::start_at(&path, ID, 2).unwrap();
    assert_eq!(loading_marker::next_retry_attempt_at(&path, ID).unwrap(), 3);
    loading_marker::start_at(&path, ID, 3).unwrap();
    assert!(loading_marker::next_retry_attempt_at(&path, ID).is_err());
}

#[test]
fn discard_invalid_refuses_to_erase_a_valid_interruption() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("extension-loading.json");
    loading_marker::start_at(&path, ID, 1).unwrap();

    assert!(loading_marker::discard_invalid_at(&path).is_err());
    assert!(matches!(
        loading_marker::read_at(&path),
        MarkerRead::Valid(_)
    ));

    std::fs::write(&path, b"corrupt").unwrap();
    loading_marker::discard_invalid_at(&path).unwrap();
    assert!(matches!(
        loading_marker::read_at(&path),
        MarkerRead::Missing
    ));
}

#[test]
fn an_unreadable_preserved_marker_does_not_turn_an_empty_sync_into_a_failure() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("extension-loading.json");
    std::fs::write(&path, b"corrupt").unwrap();
    let preserved = loading_marker::preserve_at(&path);

    loading_marker::complete_at(&path, preserved, &HashSet::new(), None).unwrap();
    assert!(matches!(
        loading_marker::read_at(&path),
        MarkerRead::Invalid
    ));
}

#[test]
fn invalid_oversized_or_unknown_marker_is_never_trusted() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("extension-loading.json");
    for bytes in [
        br#"{"version":2,"extensionId":"com.example.crash","stage":"import","startedAt":"2026-09-02T00:00:00Z","attempts":1}"#.as_slice(),
        br#"{"version":1,"extensionId":"../escape","stage":"import","startedAt":"2026-09-02T00:00:00Z","attempts":1}"#.as_slice(),
        br#"{"version":1,"extensionId":"com.example.crash","stage":"made-up","startedAt":"2026-09-02T00:00:00Z","attempts":1}"#.as_slice(),
        br#"{"version":1,"extensionId":"com.example.crash","stage":"import","startedAt":"not-a-date","attempts":1,"secret":"sentinel"}"#.as_slice(),
    ] {
        std::fs::write(&path, bytes).unwrap();
        assert!(matches!(loading_marker::read_at(&path), MarkerRead::Invalid));
    }
    std::fs::write(&path, vec![b'x'; loading_marker::MAX_MARKER_BYTES + 1]).unwrap();
    assert!(matches!(
        loading_marker::read_at(&path),
        MarkerRead::Invalid
    ));
}

#[test]
fn failed_publication_keeps_the_previous_complete_marker() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("extension-loading.json");
    loading_marker::start_at(&path, ID, 1).unwrap();

    assert!(loading_marker::advance_fail_before_replace_at(&path, ID, "activate").is_err());
    let MarkerRead::Valid(marker) = loading_marker::read_at(&path) else {
        panic!("previous marker expected");
    };
    assert_eq!(marker.stage, "import");
}

#[test]
fn marker_is_removed_only_after_its_result_is_applied() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("extension-loading.json");
    let preserved = loading_marker::preserve_at(&path);
    loading_marker::start_at(&path, ID, 1).unwrap();

    loading_marker::complete_at(&path, preserved, &HashSet::new(), None).unwrap();
    assert!(matches!(
        loading_marker::read_at(&path),
        MarkerRead::Valid(_)
    ));

    let preserved = loading_marker::preserve_at(&directory.path().join("missing.json"));
    loading_marker::complete_at(&path, preserved, &HashSet::from([ID.to_string()]), None).unwrap();
    assert!(matches!(
        loading_marker::read_at(&path),
        MarkerRead::Missing
    ));
}

#[test]
fn applied_result_without_its_expected_marker_fails_closed() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("extension-loading.json");
    let preserved = loading_marker::preserve_at(&path);

    assert!(
        loading_marker::complete_at(&path, preserved, &HashSet::from([ID.to_string()]), None,)
            .is_err()
    );
}

#[test]
fn successful_unrelated_load_restores_the_interrupted_marker() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("extension-loading.json");
    loading_marker::start_at(&path, ID, 1).unwrap();
    let preserved = loading_marker::preserve_at(&path);
    loading_marker::start_at(&path, "com.example.safe", 1).unwrap();

    loading_marker::complete_at(
        &path,
        preserved,
        &HashSet::from(["com.example.safe".to_string()]),
        None,
    )
    .unwrap();

    let MarkerRead::Valid(marker) = loading_marker::read_at(&path) else {
        panic!("interrupted marker expected");
    };
    assert_eq!(marker.extension_id, ID);
}

#[test]
fn a_failed_retry_keeps_its_incremented_attempt_after_a_neighbor_succeeds() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("extension-loading.json");
    loading_marker::start_at(&path, ID, 1).unwrap();
    let preserved = loading_marker::preserve_at(&path);
    loading_marker::start_at(&path, ID, 2).unwrap();
    loading_marker::start_at(&path, "com.example.safe", 1).unwrap();

    loading_marker::complete_at(
        &path,
        preserved,
        &HashSet::from(["com.example.safe".to_string()]),
        Some((ID, 2)),
    )
    .unwrap();

    let MarkerRead::Valid(marker) = loading_marker::read_at(&path) else {
        panic!("retry marker expected");
    };
    assert_eq!(marker.extension_id, ID);
    assert_eq!(marker.attempts, 2);
}

#[test]
fn an_oversized_marker_can_only_be_removed_by_explicit_discard() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("extension-loading.json");
    let oversized = vec![b'x'; loading_marker::MAX_MARKER_BYTES + 1];
    std::fs::write(&path, &oversized).unwrap();

    assert!(loading_marker::start_at(&path, "beaver.official.word", 1).is_err());
    assert_eq!(std::fs::read(&path).unwrap(), oversized);
    loading_marker::discard_at(&path).unwrap();
    assert!(matches!(
        loading_marker::read_at(&path),
        MarkerRead::Missing
    ));
}

#[test]
fn successful_builtin_load_restores_a_regular_corrupt_marker() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("extension-loading.json");
    std::fs::write(&path, b"corrupt-sentinel-without-path-or-url").unwrap();
    let preserved = loading_marker::preserve_at(&path);
    assert!(matches!(preserved.state, MarkerRead::Invalid));
    loading_marker::start_at(&path, "beaver.official.word", 1).unwrap();

    loading_marker::complete_at(
        &path,
        preserved,
        &HashSet::from(["beaver.official.word".to_string()]),
        None,
    )
    .unwrap();

    assert!(matches!(
        loading_marker::read_at(&path),
        MarkerRead::Invalid
    ));
    assert_eq!(
        std::fs::read(&path).unwrap(),
        b"corrupt-sentinel-without-path-or-url"
    );
}

#[cfg(unix)]
#[test]
fn marker_refuses_symbolic_links_without_touching_their_target() {
    use std::os::unix::fs::symlink;
    let directory = tempfile::tempdir().unwrap();
    let target = directory.path().join("target");
    let marker = directory.path().join("extension-loading.json");
    std::fs::write(&target, b"sentinel").unwrap();
    symlink(&target, &marker).unwrap();

    assert!(matches!(
        loading_marker::read_at(&marker),
        MarkerRead::Invalid
    ));
    assert!(loading_marker::start_at(&marker, ID, 1).is_err());
    assert!(loading_marker::discard_at(&marker).is_err());
    assert_eq!(std::fs::read(&target).unwrap(), b"sentinel");
}
