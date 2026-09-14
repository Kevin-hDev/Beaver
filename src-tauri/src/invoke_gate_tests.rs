use super::invoke_gate::{allowed, resize_allowed};

#[test]
fn invoke_gate_allows_exactly_the_seven_update_commands() {
    for command in [
        "list_update_operations",
        "dismiss_update_operation",
        "request_update_operation_retry",
        "resize_update_progress_window",
        "cancel_app_update_download",
        "cancel_ollama_setup",
        "cancel_model_download",
    ] {
        assert!(allowed("update-progress", command), "{command}");
    }
}

#[test]
fn invoke_gate_bounds_resize_to_the_update_window() {
    assert!(resize_allowed("update-progress", 96));
    assert!(resize_allowed("update-progress", 640));
    assert!(!resize_allowed("update-progress", 95));
    assert!(!resize_allowed("update-progress", 641));
    assert!(!resize_allowed("main", 200));
}

#[test]
fn invoke_gate_rejects_sensitive_and_unknown_commands_from_update_window() {
    for command in ["set_api_key", "write_agent_md", "pty_spawn", "unknown"] {
        assert!(!allowed("update-progress", command), "{command}");
    }
    assert!(allowed("main", "set_api_key"));
    assert!(allowed("main", "unknown"));
}
