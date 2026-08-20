// Filet de lisibilité uniquement : les tests comportementaux prouvent les effets,
// ceux-ci signalent qu'un refactor a déplacé une frontière visible dans la source.
#[test]
fn child_guard_runs_before_stream_replacement_and_permission_mutation() {
    let source = include_str!("agent_chat_admission.rs");
    let command = command_body(source, "pub(crate) async fn admit");
    let guard = command
        .find("session_user_write::ensure_allowed")
        .expect("admit doit appeler session_user_write::ensure_allowed");
    let permission = command
        .find("session_permission_state::prepare_send")
        .expect("admit doit préparer les permissions");
    let replacement = command
        .find("replace_active_stream")
        .expect("admit doit remplacer le stream actif");

    assert!(guard < permission);
    assert!(guard < replacement);
}

#[test]
fn child_guard_runs_before_queue_stream_lookup() {
    let source = include_str!("agent_chat_queue.rs");
    assert_guard_precedes(
        command_body(source, "pub async fn queue_agent_message"),
        "streams.0.lock",
    );
}

#[test]
fn child_guard_runs_before_each_user_session_mutation() {
    let source = include_str!("agent_sessions.rs");
    for (command, boundary) in [
        ("pub async fn save_agent_session", "session_store::get"),
        ("pub async fn rename_agent_session", "session_store::rename"),
        (
            "pub async fn set_session_permission_mode",
            "PermissionMode::parse",
        ),
        (
            "pub async fn add_messages_to_session",
            "add_messages_with_context",
        ),
        (
            "pub async fn update_session_model",
            "session_store::update_model",
        ),
        (
            "pub async fn update_session_reasoning",
            "session_store::update_reasoning",
        ),
        (
            "pub async fn set_session_plan_mode",
            "tool_plan::set_enabled",
        ),
        (
            "pub async fn truncate_and_replace_at",
            "session_store::truncate_and_replace",
        ),
    ] {
        assert_guard_precedes(command_body(source, command), boundary);
    }
}

#[test]
fn child_guard_runs_before_preflight_path_validation_and_disk_access() {
    let source = include_str!("../services/agent_local/agent_send_preflight.rs");
    assert_guard_precedes(
        command_body(source, "pub async fn prepare"),
        "session_store::get",
    );
    assert_guard_precedes(
        command_body(source, "pub async fn resolve"),
        "validate_path",
    );
}

fn command_body<'a>(source: &'a str, signature: &str) -> &'a str {
    let start = source.find(signature).expect("command signature");
    let body = &source[start..];
    &body[..body.find("\n}").map_or(body.len(), |end| end + 2)]
}

fn assert_guard_precedes(command: &str, boundary: &str) {
    let guard = command
        .find("session_user_write::ensure_allowed")
        .expect("child write guard");
    let boundary = command.find(boundary).expect("mutation boundary");
    assert!(guard < boundary, "child guard runs after {boundary}");
}
