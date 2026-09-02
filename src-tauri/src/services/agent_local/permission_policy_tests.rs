use crate::services::agent_local::permission_policy::{
    extension_effect_policy, requires_sensitive_bash_prompt, uses_auto_bypass,
};
use crate::services::extensions::ExtensionEffect;
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
    assert!(!requires_sensitive_bash_prompt("subagent", "bash", &args));
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

#[test]
fn sensitive_bash_control_input_is_redacted_before_manual_approval() {
    let args = json!({"session_id": "session", "chars": "cat ~/.ssh/id_ed25519\n"});

    assert!(requires_sensitive_bash_prompt(
        "manual",
        "bash_control",
        &args
    ));
    assert!(!requires_sensitive_bash_prompt(
        "auto",
        "bash_control",
        &args
    ));
}

#[test]
fn every_extension_effect_has_an_explicit_policy() {
    let cases = [
        (ExtensionEffect::ReadOnly, false, true, true, false),
        (ExtensionEffect::ExternalRead, true, true, false, true),
        (ExtensionEffect::LocalWrite, true, false, false, true),
        (ExtensionEffect::ExternalWrite, true, false, false, true),
        (ExtensionEffect::Process, true, false, false, false),
        (ExtensionEffect::Secret, true, false, false, false),
        (ExtensionEffect::Unknown, true, false, false, false),
    ];

    for (effect, confirm, parallel, plan, cache) in cases {
        let policy = extension_effect_policy(effect);
        assert_eq!(policy.requires_confirmation, confirm, "{effect:?}");
        assert_eq!(policy.parallel_read, parallel, "{effect:?}");
        assert_eq!(policy.allowed_in_plan, plan, "{effect:?}");
        assert_eq!(policy.allow_session_cache, cache, "{effect:?}");
    }
}
