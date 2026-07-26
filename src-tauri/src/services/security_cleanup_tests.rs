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
