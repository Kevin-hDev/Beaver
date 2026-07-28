mod builtin;
mod core_bridge;
pub(crate) mod discovery;
mod discovery_text;
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
mod types;
mod validation;
mod view;

pub use types::{ExtensionHostStatus, ExtensionKind};
pub use view::ExtensionView;

pub(crate) use discovery::{
    discover_plugin_tools, search as search_tools, select_plugin_tools, MAX_SEARCH_QUERY_CHARS,
    MAX_SEARCH_RESULTS, MAX_SELECTED_TOOLS, SEARCH_TOOL_NAME,
};
pub use registry::{add_local, disable_hosted_extensions, list, set_enabled, set_show_in_chat};
pub(crate) use registry_index::dynamic_tool_names;
pub use registry_index::{is_dynamic_tool, is_replacement};
pub use runtime::{restart, status, stop};
pub use runtime_dispatch::{dispatch_tool, emit_event};
pub use startup::initialize_on_startup;
pub use tool_bridge::{merge_definitions as merge_tool_definitions, validate_arguments};

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
