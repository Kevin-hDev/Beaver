mod extension_session_plugins;
pub mod extension_session_state;
pub mod extension_tool_mask;
pub mod extension_tool_selection;
mod extension_tool_set_apply;
mod extension_tool_set_diagnostics;
mod extension_tool_correlation;
mod extension_tool_diagnostic;
#[cfg(test)]
mod extension_tool_interception_paths_tests;
pub mod extension_skill_loader;
#[cfg(test)]
mod extension_skill_loader_tests;
mod extension_tool_authority;
pub mod extension_tool_set;
#[allow(dead_code)]
mod extension_discovery_contract {
    include!(concat!(env!("OUT_DIR"), "/extension_discovery_contract.rs"));
}
#[cfg(test)]
mod extension_discovery_contract_tests;
