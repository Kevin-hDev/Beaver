use super::update_notifications::{
    dismiss_at_path, read_from_path, DismissedUpdate, DismissedUpdateKind, MAX_DISMISSED_UPDATES,
};

fn app(version: &str) -> DismissedUpdate {
    DismissedUpdate {
        kind: DismissedUpdateKind::App,
        subject: "beaver".into(),
        version: version.into(),
    }
}

#[test]
fn missing_store_starts_empty_and_first_write_is_versioned() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("update-notifications.json");

    assert_eq!(read_from_path(&path).unwrap(), Vec::new());
    dismiss_at_path(&path, app("1.1.8")).unwrap();

    let json: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    assert_eq!(json["schema_version"], 1);
    assert_eq!(json["dismissed"][0]["version"], "1.1.8");
}

#[test]
fn newer_version_replaces_only_the_same_notification_subject() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("update-notifications.json");
    dismiss_at_path(&path, app("1.1.8")).unwrap();
    dismiss_at_path(&path, app("1.1.9")).unwrap();
    dismiss_at_path(
        &path,
        DismissedUpdate {
            kind: DismissedUpdateKind::OllamaBinary,
            subject: "ollama".into(),
            version: "0.33.1".into(),
        },
    )
    .unwrap();

    assert_eq!(
        read_from_path(&path).unwrap(),
        vec![
            app("1.1.9"),
            DismissedUpdate {
                kind: DismissedUpdateKind::OllamaBinary,
                subject: "ollama".into(),
                version: "0.33.1".into(),
            },
        ]
    );
}

#[test]
fn externally_supplied_dismissals_are_validated_and_bounded() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("update-notifications.json");
    assert!(dismiss_at_path(
        &path,
        DismissedUpdate {
            kind: DismissedUpdateKind::OllamaModel,
            subject: "../model".into(),
            version: "digest".into(),
        },
    )
    .is_err());

    for index in 0..=MAX_DISMISSED_UPDATES {
        dismiss_at_path(
            &path,
            DismissedUpdate {
                kind: DismissedUpdateKind::OllamaModel,
                subject: format!("model-{index}:latest"),
                version: format!("digest-{index}"),
            },
        )
        .unwrap();
    }

    let dismissed = read_from_path(&path).unwrap();
    assert_eq!(dismissed.len(), MAX_DISMISSED_UPDATES);
    assert_eq!(dismissed.first().unwrap().subject, "model-1:latest");
}

#[test]
fn corrupt_store_recovers_without_hiding_future_updates() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("update-notifications.json");
    std::fs::write(&path, b"not-json").unwrap();

    assert_eq!(read_from_path(&path).unwrap(), Vec::new());
    dismiss_at_path(&path, app("1.1.8")).unwrap();
    assert_eq!(read_from_path(&path).unwrap(), vec![app("1.1.8")]);
}
