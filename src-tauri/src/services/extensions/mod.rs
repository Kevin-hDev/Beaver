mod builtin;
mod core_bridge;
pub(crate) mod discovery;
mod discovery_catalog;
mod discovery_limits;
mod discovery_preferences;
mod discovery_usage;
mod error_codes;
mod git_checkout;
mod git_package;
mod git_source;
mod host_channel;
mod host_paths;
mod host_process;
mod host_reader;
mod install_preparation;
mod installer;
mod installer_record;
mod managed_cleanup;
mod managed_store;
mod managed_tree;
mod manifest;
mod manifest_source;
mod message_validation;
mod npm_environment;
mod npm_runner;
mod npm_source;
mod operation_error;
mod operation_failure;
mod operation_log;
mod origin_validation;
mod process_runner;
mod protocol;
mod registry;
mod registry_index;
mod registry_managed;
mod registry_mutation_error;
mod registry_state;
mod registry_sync;
mod runtime;
mod runtime_diagnostics;
mod runtime_dispatch;
mod runtime_restart;
mod runtime_sync;
mod runtime_version;
mod source_validation;
mod startup;
mod storage;
mod tool_bridge;
mod tool_result;
mod types;
mod validation;
mod view;

pub use types::{ExtensionHostStatus, ExtensionKind};
pub use view::ExtensionView;

pub(crate) use discovery::PluginMatch;
pub(crate) use discovery::{
    search as search_plugins, MAX_SEARCH_QUERY_CHARS, MAX_SEARCH_RESULTS, SEARCH_TOOL_NAME,
};
pub(crate) use discovery_catalog::CatalogSnapshot;
pub use discovery_preferences::DiscoveryPreferences;
pub use registry::{add_local, disable_hosted_extensions, list, set_enabled, set_show_in_chat};
pub(crate) use registry_index::{
    catalog_snapshot, dynamic_tool_names, indexed_plugins, plugin_id_for_tool,
};
pub use registry_index::{is_dynamic_tool, is_replacement};
pub use runtime::{restart, status, stop};
pub use runtime_dispatch::{dispatch_tool, emit_event};
pub use startup::initialize_on_startup;
pub(crate) use tool_bridge::definitions as extension_tool_definitions;
pub(crate) use tool_bridge::{core_fallback, without_core_fallback};
pub use tool_bridge::{merge_definitions as merge_tool_definitions, validate_arguments};
pub(crate) use tool_result::unavailable as unavailable_tool_result;

pub fn discovery_preferences() -> Result<DiscoveryPreferences, String> {
    discovery_preferences::get()
}

pub fn set_discovery_preferences(plugin_ids: Vec<String>) -> Result<DiscoveryPreferences, String> {
    discovery_preferences::set(plugin_ids)
}

pub(crate) fn record_tool_invocation(tool_name: &str) -> Result<(), String> {
    discovery_usage::record_invocation(tool_name)
}

pub(crate) const MAX_DISCOVERED_PLUGINS: usize = types::MAX_EXTENSIONS;
pub(crate) const MAX_EXTENSION_TOOLS: usize = types::MAX_TOOLS;

pub(crate) use installer::{
    install_git as install_git_source, install_npm as install_npm_source,
    uninstall as uninstall_extension, update as update_managed_extension,
};
pub(crate) use manifest::load_local as install_local;
pub(crate) use operation_error::{report as report_operation_error, Operation};
pub(crate) use operation_failure::OperationFailure;
pub(crate) use validation::identifier as validate_identifier;

#[cfg(test)]
mod builtin_tests;
#[cfg(test)]
mod git_dependencies_tests;
#[cfg(test)]
mod git_policy_tests;
#[cfg(test)]
mod git_source_reference_tests;
#[cfg(test)]
mod git_source_tests;
#[cfg(test)]
mod managed_install_error_tests;
#[cfg(test)]
mod managed_store_tests;
#[cfg(test)]
mod npm_runner_tests;
#[cfg(test)]
mod runtime_sync_tests;
#[cfg(test)]
mod source_validation_tests;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod view_tests;
