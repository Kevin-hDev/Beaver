use serde_json::Value;

use crate::services::extensions::ExtensionEffect;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExtensionEffectPolicy {
    pub requires_confirmation: bool,
    pub parallel_read: bool,
    pub allowed_in_plan: bool,
    pub allow_session_cache: bool,
}

#[cfg(test)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExtensionModeDecision {
    Allow,
    Confirm,
    Deny,
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
            allowed_in_plan: true,
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

#[cfg(test)]
pub fn extension_mode_decision(effect: ExtensionEffect, mode: &str) -> ExtensionModeDecision {
    let policy = extension_effect_policy(effect);
    match mode {
        "chat" => ExtensionModeDecision::Deny,
        "manual" if policy.requires_confirmation => ExtensionModeDecision::Confirm,
        "manual" | "auto" => ExtensionModeDecision::Allow,
        "plan" if policy.allowed_in_plan => ExtensionModeDecision::Allow,
        "plan" => ExtensionModeDecision::Deny,
        "subagent" if effect == ExtensionEffect::ReadOnly => ExtensionModeDecision::Allow,
        "subagent" => ExtensionModeDecision::Deny,
        _ => ExtensionModeDecision::Deny,
    }
}

pub fn uses_auto_bypass(mode: &str) -> bool {
    matches!(mode, "auto" | "subagent")
}

pub fn requires_sensitive_bash_prompt(
    mode: &str,
    tool_name: &str,
    args: &Value,
) -> bool {
    if uses_auto_bypass(mode) {
        return false;
    }
    let input = match tool_name {
        "bash" => args["command"].as_str(),
        "bash_control" => args["chars"].as_str(),
        _ => None,
    };
    input
        .map(super::sensitive_data::bash_touches_sensitive_data)
        .unwrap_or(false)
}

#[cfg(test)]
#[path = "permission_policy_tests.rs"]
mod tests;
