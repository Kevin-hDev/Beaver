use crate::services::extensions::ExtensionEffect;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExtensionEffectPolicy {
    pub requires_confirmation: bool,
    pub parallel_read: bool,
    pub allowed_in_plan: bool,
    pub allow_session_cache: bool,
}

pub fn extension_effect_policy(effect: ExtensionEffect) -> ExtensionEffectPolicy {
    use ExtensionEffect::*;
    match effect {
        ReadOnly => ExtensionEffectPolicy {
            requires_confirmation: false,
            parallel_read: true,
            allowed_in_plan: true,
            allow_session_cache: false,
        },
        ExternalRead => ExtensionEffectPolicy {
            requires_confirmation: true,
            parallel_read: true,
            allowed_in_plan: false,
            allow_session_cache: true,
        },
        LocalWrite | ExternalWrite => ExtensionEffectPolicy {
            requires_confirmation: true,
            parallel_read: false,
            allowed_in_plan: false,
            allow_session_cache: true,
        },
        Process | Secret | Unknown => ExtensionEffectPolicy {
            requires_confirmation: true,
            parallel_read: false,
            allowed_in_plan: false,
            allow_session_cache: false,
        },
    }
}

pub fn uses_auto_bypass(mode: &str) -> bool {
    matches!(mode, "auto" | "subagent")
}

#[cfg(test)]
#[path = "permission_policy_tests.rs"]
mod tests;
