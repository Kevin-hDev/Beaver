use super::{
    load_or_migrate, parse_policy, write_policy, ForecastSelectionMode, ForecastSelectionPolicy,
    MAX_POLICY_BYTES,
};

const MODEL_ID: &str = "chronos-bolt-small";

#[test]
fn missing_files_create_a_manual_local_only_policy() {
    let dir = tempfile::tempdir().unwrap();
    let policy_path = dir.path().join("policy.json");
    let legacy_path = dir.path().join("legacy.json");

    let policy = load_or_migrate(&policy_path, &legacy_path).unwrap();

    assert_eq!(policy, ForecastSelectionPolicy::default());
    assert!(policy_path.exists());
}

#[test]
fn legacy_selection_migrates_without_enabling_auto_or_cloud() {
    let dir = tempfile::tempdir().unwrap();
    let policy_path = dir.path().join("policy.json");
    let legacy_path = dir.path().join("legacy.json");
    std::fs::write(&legacy_path, format!(r#"{{"model":"{MODEL_ID}"}}"#)).unwrap();

    let policy = load_or_migrate(&policy_path, &legacy_path).unwrap();

    assert_eq!(policy.mode, ForecastSelectionMode::Manual);
    assert_eq!(policy.manual_model_id.as_deref(), Some(MODEL_ID));
    assert!(!policy.allow_cloud_in_auto);
    assert!(legacy_path.exists());
}

#[test]
fn auto_round_trip_preserves_the_last_manual_model() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("policy.json");
    let policy = ForecastSelectionPolicy {
        mode: ForecastSelectionMode::Auto,
        manual_model_id: Some(MODEL_ID.to_string()),
        allow_cloud_in_auto: false,
    };

    write_policy(&path, &policy).unwrap();
    let restored = parse_policy(&std::fs::read(path).unwrap()).unwrap();

    assert_eq!(restored, policy);
}

#[test]
fn invalid_legacy_model_fails_closed_without_creating_policy() {
    let dir = tempfile::tempdir().unwrap();
    let policy_path = dir.path().join("policy.json");
    let legacy_path = dir.path().join("legacy.json");
    std::fs::write(&legacy_path, r#"{"model":"../../unsafe"}"#).unwrap();

    assert!(load_or_migrate(&policy_path, &legacy_path).is_err());
    assert!(!policy_path.exists());
}

#[test]
fn invalid_policy_is_not_silently_replaced() {
    let dir = tempfile::tempdir().unwrap();
    let policy_path = dir.path().join("policy.json");
    let legacy_path = dir.path().join("legacy.json");
    std::fs::write(&policy_path, b"not-json").unwrap();

    assert!(load_or_migrate(&policy_path, &legacy_path).is_err());
}

#[test]
fn oversized_policy_is_rejected_before_unbounded_reading() {
    let dir = tempfile::tempdir().unwrap();
    let policy_path = dir.path().join("policy.json");
    let legacy_path = dir.path().join("legacy.json");
    std::fs::write(&policy_path, vec![b'x'; MAX_POLICY_BYTES + 1]).unwrap();

    assert!(load_or_migrate(&policy_path, &legacy_path).is_err());
}
