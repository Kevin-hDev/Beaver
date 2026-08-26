use super::*;
use std::fs;
use tempfile::TempDir;

fn write(path: &Path, content: &[u8]) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(path, content).unwrap();
}

#[test]
fn removes_only_legacy_files_and_sanitizes_sessions() {
    let root = TempDir::new().unwrap();
    let old_backup = root.path().join("secrets.enc.bak-corrupted");
    let old_kimi = root
        .path()
        .join("oauth-providers/moonshot/credentials/kimi-code.json");
    let old_xai = root.path().join("oauth-providers/xai/auth.json");
    let current_vault = root.path().join("secrets.enc");
    let current_device = root.path().join("oauth-providers/kimi-device-id");
    let session = root.path().join("agent-sessions/session.json");

    write(&old_backup, b"encrypted backup");
    write(&old_kimi, br#"{"access_token":"legacy-kimi-token"}"#);
    write(&old_xai, br#"{"access_token":"legacy-xai-token"}"#);
    write(&current_vault, b"current encrypted vault");
    write(&current_device, b"current device identifier");
    write(
        &session,
        br#"{"messages":[{"content":"gsk_1234567890abcdefghijkl"}]}"#,
    );

    run_in(root.path()).unwrap();

    assert!(!old_backup.exists());
    assert!(!old_kimi.exists());
    assert!(!old_xai.exists());
    assert!(current_vault.exists());
    assert!(current_device.exists());
    let cleaned = fs::read_to_string(&session).unwrap();
    assert!(!cleaned.contains("gsk_1234567890abcdefghijkl"));
    assert!(cleaned.contains("[REDACTED]"));
    assert!(root.path().join(MARKER_FILE).exists());

    run_in(root.path()).unwrap();
    assert!(current_vault.exists());
}

#[cfg(unix)]
#[test]
fn rewritten_sessions_are_private() {
    use std::os::unix::fs::PermissionsExt;

    let root = TempDir::new().unwrap();
    let session = root.path().join("agent-sessions/session.json");
    write(&session, br#"{"content":"token=private-value"}"#);
    fs::set_permissions(&session, fs::Permissions::from_mode(0o644)).unwrap();

    run_in(root.path()).unwrap();

    assert_eq!(
        fs::metadata(session).unwrap().permissions().mode() & 0o777,
        0o600
    );
}

#[test]
fn cleanup_redacts_visible_text_without_mutating_continuation_or_provider_ids() {
    let root = TempDir::new().unwrap();
    let session = root.path().join("agent-sessions/session.json");
    let tool_extra = serde_json::json!({
        "google": {"thought_signature": "Bearer opaque-tool-signature-12345678"},
        "codex": {"output_items": [{"id": "sk-output-item-12345678"}]}
    });
    let controlled_collisions = serde_json::json!({
        "id": "sk-controlled-id-12345678",
        "continuation": "Bearer controlled-continuation-12345678",
        "extra_content": "aaaaaaaaaaaaaaaaaaaa.bbbbb.cccccccccccccccccccc",
        "provider_id": "sk-controlled-provider-id-12345678"
    });
    let continuation = serde_json::json!({
        "schema_version": 1,
        "continuation": {
            "type": "responses_local",
            "items": [{
                "encrypted_content": "Bearer opaque-native-token-12345678",
                "provider_item_id": "sk-native-item-12345678"
            }]
        }
    });
    write(
        &session,
        &serde_json::to_vec_pretty(&serde_json::json!({
            "messages": [{
                "id": "sk-message-id-12345678",
                "turn_id": "sk-turn-id-12345678",
                "tool_call_id": "sk-linked-call-id-12345678",
                "role": "assistant",
                "content": "sk-visible-content-12345678",
                "tool_calls": [{
                    "id": "sk-provider-call-12345678",
                    "extra_content": tool_extra,
                    "function": {
                        "name":"read_file",
                        "arguments": controlled_collisions
                    }
                }],
                "tool_activities": [{
                    "name": "read_file",
                    "args": controlled_collisions,
                    "result": controlled_collisions.to_string()
                }],
                "continuation": continuation
            }]
        }))
        .unwrap(),
    );

    run_in(root.path()).unwrap();
    let restored: serde_json::Value = serde_json::from_slice(&fs::read(session).unwrap()).unwrap();

    assert_eq!(restored["messages"][0]["content"], "[REDACTED]");
    assert_eq!(restored["messages"][0]["id"], "sk-message-id-12345678");
    assert_eq!(restored["messages"][0]["turn_id"], "sk-turn-id-12345678");
    assert_eq!(
        restored["messages"][0]["tool_call_id"],
        "sk-linked-call-id-12345678"
    );
    assert_eq!(
        restored["messages"][0]["tool_calls"][0]["id"],
        "sk-provider-call-12345678"
    );
    assert_eq!(
        restored["messages"][0]["tool_calls"][0]["extra_content"],
        serde_json::json!({ "google": tool_extra["google"].clone() })
    );
    assert_eq!(restored["messages"][0]["continuation"], continuation);
    for key in ["id", "continuation", "extra_content", "provider_id"] {
        assert_eq!(
            restored["messages"][0]["tool_calls"][0]["function"]["arguments"][key],
            "[REDACTED]"
        );
        assert_eq!(
            restored["messages"][0]["tool_activities"][0]["args"][key],
            "[REDACTED]"
        );
    }
    let result = restored["messages"][0]["tool_activities"][0]["result"]
        .as_str()
        .unwrap();
    for secret in [
        "sk-controlled-id-12345678",
        "controlled-continuation-12345678",
        "aaaaaaaaaaaaaaaaaaaa.bbbbb.cccccccccccccccccccc",
        "sk-controlled-provider-id-12345678",
    ] {
        assert!(!result.contains(secret));
    }
    assert!(result.contains("[REDACTED]"));
}

#[test]
fn refuses_to_delete_a_directory_at_a_legacy_file_path() {
    let root = TempDir::new().unwrap();
    let unexpected = root.path().join("secrets.enc.bak-corrupted");
    fs::create_dir_all(&unexpected).unwrap();

    assert!(run_in(root.path()).is_err());
    assert!(unexpected.exists());
    assert!(!root.path().join(MARKER_FILE).exists());
}

#[test]
fn ignores_a_directory_that_only_looks_like_a_session_file() {
    let root = TempDir::new().unwrap();
    let unexpected = root.path().join("agent-sessions/not-a-session.json");
    fs::create_dir_all(&unexpected).unwrap();

    run_in(root.path()).unwrap();

    assert!(unexpected.is_dir());
    assert!(root.path().join(MARKER_FILE).is_file());
}

#[test]
fn removes_orphan_backup_even_after_hardening_marker_exists() {
    let root = TempDir::new().unwrap();
    run_in(root.path()).unwrap();
    let backup = root
        .path()
        .join("agent-sessions/550e8400-e29b-41d4-a716-446655440000.json.v1.bak");
    write(&backup, b"private fixture backup");

    run_in(root.path()).unwrap();

    assert!(!backup.exists());
}

#[test]
fn keeps_backup_attached_to_regular_session() {
    let root = TempDir::new().unwrap();
    let main = root
        .path()
        .join("agent-sessions/550e8400-e29b-41d4-a716-446655440000.json");
    let backup = main.with_file_name("550e8400-e29b-41d4-a716-446655440000.json.v1.bak");
    write(&main, br#"{"messages":[]}"#);
    write(&backup, b"private fixture backup");

    run_in(root.path()).unwrap();

    assert!(backup.is_file());
}

#[test]
fn ignores_noncanonical_or_wrong_type_backup_without_blocking_hardening() {
    let root = TempDir::new().unwrap();
    let invalid = root.path().join("agent-sessions/not-valid!.json.v1.bak");
    write(&invalid, b"fixture");
    assert!(run_in(root.path()).is_ok());

    fs::remove_file(&invalid).unwrap();
    let directory = root
        .path()
        .join("agent-sessions/550e8400-e29b-41d4-a716-446655440000.json.v1.bak");
    fs::create_dir_all(directory).unwrap();
    assert!(run_in(root.path()).is_ok());
}

#[cfg(unix)]
#[test]
fn ignores_symbolic_backup_without_following_it() {
    use std::os::unix::fs::symlink;

    let root = TempDir::new().unwrap();
    let directory = root.path().join("agent-sessions");
    fs::create_dir_all(&directory).unwrap();
    let target = root.path().join("target");
    write(&target, b"fixture");
    let backup = directory.join("550e8400-e29b-41d4-a716-446655440000.json.v1.bak");
    symlink(target, backup).unwrap();

    assert!(run_in(root.path()).is_ok());
    assert_eq!(fs::read(root.path().join("target")).unwrap(), b"fixture");
}

#[test]
fn refuses_to_scan_more_than_the_central_session_file_limit() {
    let root = TempDir::new().unwrap();
    let directory = root.path().join("agent-sessions");
    fs::create_dir_all(&directory).unwrap();
    for index in 0..=crate::services::agent_local::session_limits::MAX_SESSION_FILES {
        write(&directory.join(format!("ignored-{index}.txt")), b"");
    }

    assert!(run_in(root.path()).is_err());
    assert!(!root.path().join(MARKER_FILE).exists());
}
