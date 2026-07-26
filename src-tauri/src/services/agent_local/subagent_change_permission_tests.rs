use super::permission_gate::requires_permission;
use serde_json::json;

#[test]
fn applying_subagent_changes_requires_manual_permission() {
    let args = json!({
        "subagent_id": uuid::Uuid::new_v4().to_string(),
        "change_id": uuid::Uuid::new_v4().to_string(),
    });

    assert!(requires_permission("apply_subagent_changes", &args));
}

#[test]
fn inspecting_or_discarding_subagent_changes_needs_no_permission() {
    let args = json!({});

    assert!(!requires_permission("inspect_subagent_changes", &args));
    assert!(!requires_permission("discard_subagent_changes", &args));
}
