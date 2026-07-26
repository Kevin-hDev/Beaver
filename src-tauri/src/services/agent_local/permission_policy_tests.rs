use crate::services::agent_local::permission_policy::{
    requires_sensitive_bash_prompt, uses_auto_bypass,
};
use serde_json::json;

#[test]
fn only_full_access_modes_bypass_permissions() {
    assert!(uses_auto_bypass("auto"));
    assert!(uses_auto_bypass("subagent"));
    assert!(!uses_auto_bypass("manual"));
    assert!(!uses_auto_bypass("chat"));
}

#[test]
fn full_access_never_prompts_for_sensitive_bash() {
    let args = json!({"command": "cat ~/.ssh/id_ed25519"});

    assert!(!requires_sensitive_bash_prompt("auto", "bash", &args));
    assert!(!requires_sensitive_bash_prompt(
        "subagent", "bash", &args
    ));
    assert!(requires_sensitive_bash_prompt("manual", "bash", &args));
}

#[test]
fn manual_mode_does_not_apply_the_sensitive_bash_rule_to_other_tools() {
    let args = json!({"path": "~/.ssh/id_ed25519"});

    assert!(!requires_sensitive_bash_prompt(
        "manual",
        "read_file",
        &args
    ));
}
