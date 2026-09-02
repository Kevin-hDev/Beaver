use crate::services::agent_local::permission_policy::{
    extension_effect_policy, extension_mode_decision, requires_sensitive_bash_prompt,
    uses_auto_bypass, ExtensionModeDecision,
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
        (ExtensionEffect::ExternalRead, true, true, true, true),
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

#[test]
fn extension_effect_mode_matrix_is_exhaustive() {
    use ExtensionModeDecision::{Allow, Confirm, Deny};

    let effects = ExtensionEffect::ALL;
    let modes = ["chat", "manual", "auto", "plan", "subagent"];
    let expected = [
        [Deny, Allow, Allow, Allow, Allow],
        [Deny, Confirm, Allow, Deny, Deny],
        [Deny, Confirm, Allow, Allow, Deny],
        [Deny, Confirm, Allow, Deny, Deny],
        [Deny, Confirm, Allow, Deny, Deny],
        [Deny, Confirm, Allow, Deny, Deny],
        [Deny, Confirm, Allow, Deny, Deny],
    ];

    for (effect_index, effect) in effects.into_iter().enumerate() {
        for (mode_index, mode) in modes.into_iter().enumerate() {
            assert_eq!(
                extension_mode_decision(effect, mode),
                expected[effect_index][mode_index],
                "effect={effect:?} mode={mode}",
            );
        }
    }
}

#[test]
fn office_effects_and_replacements_follow_effect_not_tool_name() {
    assert_eq!(
        extension_mode_decision(ExtensionEffect::ReadOnly, "manual"),
        ExtensionModeDecision::Allow,
    );
    assert_eq!(
        extension_mode_decision(ExtensionEffect::LocalWrite, "manual"),
        ExtensionModeDecision::Confirm,
    );
    // Un remplacement nommé read_file reste une écriture externe : le nom natif
    // ne peut jamais abaisser la classe fournie par le registre validé.
    assert_eq!(
        extension_mode_decision(ExtensionEffect::ExternalWrite, "manual"),
        ExtensionModeDecision::Confirm,
    );
}
