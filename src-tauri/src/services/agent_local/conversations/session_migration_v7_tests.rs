use std::path::PathBuf;

fn v6(owner: serde_json::Value) -> Vec<u8> {
    serde_json::to_vec(&serde_json::json!({
        "schema_version": 6,
        "id": "00000000-0000-4000-8000-000000000001",
        "name": "legacy child",
        "created_at": "2026-09-16T00:00:00Z",
        "model": "model",
        "accumulated_tokens": 0,
        "messages": [],
        "parent_session_id": "00000000-0000-4000-8000-000000000002",
        "subagent_extension_owner": owner
    }))
    .unwrap()
}

#[test]
fn session_v7_preserves_v6_artifacts_and_future_readability() {
    let loaded =
        super::session_migration::read(&v6(serde_json::Value::Null), PathBuf::from("session.json"))
            .unwrap();
    assert_eq!(loaded.session().schema_version, 7);
    assert!(loaded.session().subagent_extension_owner.is_none());

    let future =
        String::from_utf8(super::session_migration::serialize_current(loaded.session()).unwrap())
            .unwrap()
            .replace("\"schema_version\": 7", "\"schema_version\": 8");
    let future =
        super::session_migration::read(future.as_bytes(), PathBuf::from("future.json")).unwrap();
    assert_eq!(future.session().schema_version, 8);
}

#[tokio::test]
async fn v6_migration_keeps_an_exact_versioned_backup() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("session.json");
    let source = v6(serde_json::Value::Null);
    crate::services::private_store::atomic_write(&path, &source).unwrap();

    let loaded = super::session_migration::read(&source, path.clone()).unwrap();
    assert_eq!(
        loaded.version(),
        super::session_migration::LoadedVersion::Legacy(6)
    );
    super::session_store_document::write_to_path(path.clone(), loaded.session())
        .await
        .unwrap();

    assert_eq!(
        std::fs::read(super::session_migration::backup_path(&path, 6).unwrap()).unwrap(),
        source
    );
    let current = std::fs::read(&path).unwrap();
    assert_eq!(
        super::session_migration::read(&current, path)
            .unwrap()
            .version(),
        super::session_migration::LoadedVersion::Current
    );
}

#[test]
fn malformed_extension_owner_keeps_history_readable_but_unowned() {
    let malformed_v7 = String::from_utf8(v6(serde_json::Value::Null))
        .unwrap()
        .replace("\"schema_version\":6", "\"schema_version\":7")
        .replace(
            "\"subagent_extension_owner\":null",
            "\"subagent_extension_owner\":{\"extensionId\":[\"invalid\"]}",
        );
    let loaded =
        super::session_migration::read(malformed_v7.as_bytes(), PathBuf::from("session.json"))
            .unwrap();
    assert!(matches!(
        loaded.session().subagent_extension_owner,
        Some(super::types_session::SubagentExtensionOwnership::Invalid(_))
    ));
    let serialized = super::session_migration::serialize_current(loaded.session()).unwrap();
    let value: serde_json::Value = serde_json::from_slice(&serialized).unwrap();
    assert_eq!(
        value["subagent_extension_owner"],
        serde_json::json!({"extensionId": ["invalid"]})
    );
}
