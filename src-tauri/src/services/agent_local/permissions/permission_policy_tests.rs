use crate::services::agent_local::permission_policy::{extension_effect_policy, uses_auto_bypass};
use crate::services::extensions::ExtensionEffect;

#[test]
fn only_full_access_modes_bypass_permissions() {
    assert!(uses_auto_bypass("auto"));
    assert!(uses_auto_bypass("subagent"));
    assert!(!uses_auto_bypass("manual"));
    assert!(!uses_auto_bypass("chat"));
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
