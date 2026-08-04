use std::fs;

use super::{acknowledge_in, UPDATE_HEALTH_ARG};

fn token(index: u8) -> String {
    format!("{index:02x}").repeat(32)
}

#[test]
fn writes_ack_only_for_one_valid_internal_argument() {
    let root = tempfile::tempdir().unwrap();
    let valid = token(1);
    acknowledge_in(["app", UPDATE_HEALTH_ARG, valid.as_str()], root.path()).unwrap();
    assert_eq!(
        fs::read(
            root.path()
                .join("update-health")
                .join(format!("{valid}.ok"))
        )
        .unwrap(),
        b"ok"
    );

    for args in [
        vec!["app", UPDATE_HEALTH_ARG],
        vec!["app", UPDATE_HEALTH_ARG, "ABC"],
        vec!["app", UPDATE_HEALTH_ARG, valid.as_str(), UPDATE_HEALTH_ARG],
    ] {
        assert!(acknowledge_in(args, root.path()).is_err());
    }
}

#[test]
fn ignores_normal_launch_and_keeps_at_most_eight_ack_files() {
    let root = tempfile::tempdir().unwrap();
    acknowledge_in(["app", "--clgo-autostart"], root.path()).unwrap();
    assert!(!root.path().join("update-health").exists());

    for index in 0..12 {
        let value = token(index);
        acknowledge_in(["app", UPDATE_HEALTH_ARG, value.as_str()], root.path()).unwrap();
    }
    let count = fs::read_dir(root.path().join("update-health"))
        .unwrap()
        .count();
    assert_eq!(count, 8);
}

#[test]
fn refuses_an_unbounded_argument_list() {
    let args =
        std::iter::once("app".to_string()).chain((0..40).map(|index| format!("--arg-{index}")));
    let root = tempfile::tempdir().unwrap();
    assert!(acknowledge_in(args, root.path()).is_err());
}

#[test]
fn rejects_ambiguous_health_arguments_without_writing_an_ack() {
    let valid = token(10);
    let second = token(11);
    let uppercase = valid.to_ascii_uppercase();
    let mut oversized = vec!["app".to_string()];
    oversized.extend((0..32).map(|index| format!("--arg-{index}")));
    let cases = [
        vec![
            "app".to_string(),
            UPDATE_HEALTH_ARG.to_string(),
            valid.clone(),
            UPDATE_HEALTH_ARG.to_string(),
            second,
        ],
        vec![
            "app".to_string(),
            UPDATE_HEALTH_ARG.to_string(),
            valid,
            UPDATE_HEALTH_ARG.to_string(),
        ],
        vec!["app".to_string(), UPDATE_HEALTH_ARG.to_string(), uppercase],
        oversized,
    ];
    let mut expected_error = None;

    for args in cases {
        let root = tempfile::tempdir().unwrap();
        let error = acknowledge_in(args, root.path()).unwrap_err();
        assert!(!error.is_empty() && error.len() <= 128);
        if let Some(expected) = &expected_error {
            assert_eq!(&error, expected);
        } else {
            expected_error = Some(error);
        }
        assert!(!root.path().join("update-health").exists());
    }
}

#[cfg(unix)]
#[test]
fn refuses_a_symlinked_health_directory() {
    use std::os::unix::fs::symlink;

    let root = tempfile::tempdir().unwrap();
    let outside = tempfile::tempdir().unwrap();
    symlink(outside.path(), root.path().join("update-health")).unwrap();
    let value = token(42);
    assert!(acknowledge_in(["app", UPDATE_HEALTH_ARG, value.as_str()], root.path()).is_err());
    assert_eq!(outside.path().read_dir().unwrap().count(), 0);
}
