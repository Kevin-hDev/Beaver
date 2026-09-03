mod access_log;
mod bounded_jsonl;
mod builtin;
mod call_context;
mod core_bridge;
mod core_secrets;
pub(crate) mod discovery;
mod discovery_catalog;
mod discovery_limits;
mod discovery_preferences;
mod discovery_usage;
pub(crate) mod error_codes;
pub(crate) mod extension_recovery;
mod fingerprint;
mod fingerprint_paths;
mod git_checkout;
mod git_package;
mod git_source;
mod host_channel;
mod host_core_call;
mod host_identity;
mod host_load_tracker;
mod host_paths;
mod host_process;
mod host_reader;
mod host_reader_line;
mod host_stop_boundary;
#[cfg(test)]
mod host_stop_boundary_tests;
mod install_preparation;
mod installer;
mod installer_record;
mod installer_uninstall;
mod loading_journal_format;
mod loading_journal_store;
#[cfg(test)]
mod loading_journal_tests;
pub(crate) mod loading_marker;
mod loading_marker_format;
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
mod process_environment;
mod process_runner;
mod protocol;
mod public_api;
mod registry;
mod registry_access;
mod registry_failure;
#[cfg(test)]
mod registry_failure_tests;
mod registry_index;
mod registry_interruption;
mod registry_managed;
mod registry_memory;
mod registry_mutation_error;
pub(crate) mod registry_recovery;
mod registry_state;
mod registry_sync;
mod runtime;
mod runtime_channel_ensure;
mod runtime_channel_sync;
mod runtime_diagnostics;
mod runtime_dispatch;
mod runtime_exit_monitor;
mod runtime_failed_spawn;
mod runtime_host_generation;
mod runtime_host_load;
mod runtime_host_storage;
mod runtime_hosts;
mod runtime_lifecycle;
mod runtime_plan;
mod runtime_recovery_preflight;
mod runtime_restart;
mod runtime_sync;
mod runtime_sync_apply;
#[cfg(test)]
mod runtime_sync_apply_tests;
mod runtime_sync_contributions;
mod runtime_version;
mod source_validation;
mod startup;
mod storage;
mod storage_format;
mod storage_migration;
mod tool_bridge;
mod tool_result;
pub(crate) mod types;
mod ui_action_result;
mod ui_catalog;
mod ui_catalog_actions;
mod ui_catalog_lifecycle;
mod ui_catalog_limits;
mod ui_dispatch;
mod ui_normalization;
pub(crate) mod ui_protocol;
mod ui_types;
mod ui_validation;
mod ui_view_validation;
mod validation;
mod view;
mod work_supervision;
#[allow(dead_code)]
mod ui_contract {
    include!(concat!(env!("OUT_DIR"), "/extension_ui_contract.rs"));
}
mod ui_startup;
mod ui_startup_ack;
mod ui_startup_platform;
mod ui_startup_state;
#[cfg(test)]
mod ui_startup_tests;
#[cfg(test)]
mod work_supervision_tests;

#[cfg(test)]
mod contract_artifact_tests;
#[cfg(test)]
mod ui_contract_tests;
#[cfg(test)]
mod ui_dispatch_tests;
#[cfg(test)]
mod ui_limit_tests;
#[cfg(test)]
mod ui_protocol_tests;
#[cfg(test)]
mod ui_validation_tests;

pub use extension_recovery::ExtensionRecoveryState;
pub use types::{ExtensionEffect, ExtensionHostStatus, ExtensionKind};
pub use ui_types::{UiActionPayload, UiCatalogSnapshot};
pub use view::ExtensionView;

pub(crate) use discovery::PluginMatch;
pub(crate) use discovery::{
    search as search_plugins, MAX_SEARCH_QUERY_CHARS, MAX_SEARCH_RESULTS, SEARCH_TOOL_NAME,
};
pub(crate) use discovery_catalog::CatalogSnapshot;
pub use discovery_preferences::DiscoveryPreferences;
pub use public_api::{
    discovery_preferences, invoke_ui_action, set_discovery_preferences, ui_catalog,
};
pub use registry::{add_local, list, set_enabled, set_show_in_chat};
pub(crate) use registry_index::{
    catalog_snapshot, dynamic_tool_names, indexed_plugins, indexed_tool, plugin_id_for_tool,
};
pub use registry_index::{is_dynamic_tool, is_replacement};
pub use registry_recovery::{disable_hosted_extensions, restore_recovery_snapshot};
pub use runtime::status;
pub use runtime_dispatch::{dispatch_tool, emit_event};
pub(crate) use runtime_lifecycle::{new_stop_deadline, CHANGED_EVENT};
pub use runtime_lifecycle::{restart, stop_and_wait};
pub use startup::initialize_on_startup;
pub(crate) use tool_bridge::definitions as extension_tool_definitions;
pub(crate) use tool_bridge::{core_fallback, without_core_fallback};
pub use tool_bridge::{merge_definitions as merge_tool_definitions, validate_arguments};
pub(crate) use tool_result::unavailable as unavailable_tool_result;

pub(crate) use public_api::{
    close_command_error, record_tool_invocation, revoke_extension, MAX_DISCOVERED_PLUGINS,
    MAX_EXTENSION_TOOLS, MAX_PERMISSION_SUMMARY_CHARS,
};

pub(crate) use installer::{
    install_git as install_git_source, install_npm as install_npm_source,
    uninstall as uninstall_extension, update as update_managed_extension,
};
pub(crate) use manifest::load_local as install_local;
pub(crate) use operation_error::{report as report_operation_error, Operation};
pub(crate) use operation_failure::OperationFailure;
pub(crate) use ui_startup::prepare as prepare_ui_startup;
#[cfg(target_os = "windows")]
pub(crate) use ui_startup::{
    cef_child_safe_mode_action, cef_safe_mode_switch_name, SAFE_MODE_SWITCH,
};
pub(crate) use ui_startup_ack::{UiAckToken, UiLoadAcknowledger};
pub(crate) use ui_startup_state::{SafeReason, UiStartupMode, UiStartupState};
pub(crate) use validation::identifier as validate_identifier;

#[cfg(test)]
mod access_log_tests;
#[cfg(test)]
mod bounded_jsonl_tests;
#[cfg(test)]
mod builtin_tests;
#[cfg(test)]
mod fingerprint_tests;
#[cfg(test)]
mod git_dependencies_tests;
#[cfg(test)]
mod git_policy_tests;
#[cfg(test)]
mod git_source_reference_tests;
#[cfg(test)]
mod git_source_tests;
#[cfg(test)]
mod loading_marker_tests;
#[cfg(test)]
mod managed_install_error_tests;
#[cfg(test)]
mod managed_store_tests;
#[cfg(test)]
mod npm_runner_tests;
#[cfg(test)]
mod runtime_hosts_tests;
#[cfg(test)]
mod runtime_sync_tests;
#[cfg(test)]
mod source_validation_tests;
#[cfg(test)]
mod storage_migration_tests;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod view_tests;
