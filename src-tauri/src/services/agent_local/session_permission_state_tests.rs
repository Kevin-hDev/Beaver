use super::session_permission_state::{PermissionFamily, PermissionMode};

#[test]
fn chat_family_rejects_tool_modes() {
    assert!(PermissionFamily::Chat.allows(PermissionMode::Chat));
    assert!(!PermissionFamily::Chat.allows(PermissionMode::Manual));
    assert!(!PermissionFamily::Chat.allows(PermissionMode::Auto));
}

#[test]
fn tools_family_allows_only_manual_and_auto() {
    assert!(!PermissionFamily::Tools.allows(PermissionMode::Chat));
    assert!(PermissionFamily::Tools.allows(PermissionMode::Manual));
    assert!(PermissionFamily::Tools.allows(PermissionMode::Auto));
}

#[tokio::test]
async fn first_send_locks_family_and_persists_mode() {
    let session = super::session_store::create_full("Mode", "model", "provider", false, None)
        .await
        .expect("session");
    let mode = super::session_permission_state::prepare_send(&session.id, Some("manual"))
        .await
        .expect("prepare");
    assert_eq!(mode, "manual");
    let state = super::session_permission_state::load(&session.id)
        .await
        .expect("state");
    assert_eq!(state.permission_family, Some(PermissionFamily::Tools));
    assert_eq!(state.permission_mode, PermissionMode::Manual);
    assert!(super::session_permission_state::set_mode(&session.id, PermissionMode::Chat)
        .await
        .is_err());
    assert!(super::session_permission_state::set_mode(&session.id, PermissionMode::Auto)
        .await
        .is_ok());
    let raw = tokio::fs::read_to_string(
        crate::services::paths::data_dir()
            .join("agent-sessions")
            .join(format!("{}.json", session.id)),
    )
    .await
    .expect("session json");
    assert!(raw.contains("\"permission_family\": \"tools\""));
    assert!(raw.contains("\"permission_mode\": \"auto\""));
    super::session_store::delete_one(&session.id)
        .await
        .expect("cleanup");
}

#[tokio::test]
async fn chatbot_first_send_permanently_rejects_tool_family() {
    let session = super::session_store::create_full("Chat", "model", "provider", false, None)
        .await
        .expect("session");
    super::session_permission_state::prepare_send(&session.id, Some("chat"))
        .await
        .expect("lock chat");

    assert!(super::session_permission_state::set_mode(&session.id, PermissionMode::Manual)
        .await
        .is_err());
    assert!(super::session_permission_state::prepare_send(&session.id, Some("auto"))
        .await
        .is_err());
    super::session_store::delete_one(&session.id)
        .await
        .expect("cleanup");
}
