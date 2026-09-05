mod access_log;
mod bounded_jsonl;
mod builtin;
mod call_context;
mod contribution_path;
mod contribution_resources;
#[cfg(test)]
mod contribution_resources_tests;
mod contribution_skills;
#[cfg(test)]
mod contribution_skills_tests;
mod contribution_types;
mod core_bridge;
mod core_response_audit;
#[cfg(test)]
mod core_response_audit_tests;
mod core_secrets;
mod diagnostic_time;
mod discovery_catalog;
mod discovery_inspection;
mod discovery_limits;
mod discovery_listing;
mod discovery_preferences;
mod discovery_result_serialization;
mod discovery_usage;
pub(crate) mod error_codes;
mod extension_internal_exports;
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
mod resource_identifier;
mod resource_loader;
mod resource_loader_prepare;
mod runtime;
mod runtime_channel_ensure;
mod runtime_channel_sync;
mod runtime_diagnostics;
mod runtime_dispatch;
mod runtime_dispatch_result;
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
mod runtime_ui_diagnostics;
mod runtime_version;
mod source_validation;
mod startup;
mod storage;
mod storage_format;
mod storage_migration;
mod tool_bridge;
mod tool_result;
mod tool_result_contract;
mod tool_result_files;
mod tool_result_media;
pub(crate) mod types;
mod ui_action_result;
mod ui_artifact;
mod ui_artifact_manifest;
mod ui_artifact_store;
mod ui_build_api;
mod ui_builder;
mod ui_builder_process;
mod ui_catalog;
mod ui_catalog_actions;
mod ui_catalog_lifecycle;
mod ui_catalog_limits;
mod ui_dispatch;
mod ui_normalization;
pub(crate) mod ui_protocol;
mod ui_protocol_proof;
mod ui_protocol_response;
mod ui_types;
mod ui_validation;
mod ui_view_validation;
mod validation;
mod verified_file_read;
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
#[allow(dead_code)]
mod discovery_contract {
    include!(concat!(env!("OUT_DIR"), "/extension_discovery_contract.rs"));
}
#[cfg(test)]
mod ui_startup_tests;
#[cfg(test)]
mod work_supervision_tests;

#[cfg(test)]
include!("test_modules.inc.rs");

pub use extension_recovery::ExtensionRecoveryState;
pub use types::{ExtensionEffect, ExtensionHostStatus, ExtensionKind};
pub use ui_types::{UiActionPayload, UiCatalogSnapshot};
pub use view::ExtensionView;

pub(crate) use discovery_catalog::CatalogSnapshot;
pub(crate) use discovery_inspection::inspect as inspect_discoverable;
pub(crate) use discovery_inspection::InspectionStatus;
pub(crate) use discovery_listing::list as list_discoverable;
pub(crate) use discovery_result_serialization::serialize_bounded_result;
pub(crate) const MAX_INSPECTED_EXTENSIONS: usize = discovery_contract::MAX_INSPECTED_EXTENSIONS;
pub(crate) use discovery_contract::{CONTEXT_THRESHOLD_PERCENT, UNKNOWN_CONTEXT_TOKENS};
pub(crate) use discovery_limits::DISCOVERY_STORE_MAX_BYTES;
pub(crate) const MAX_COMPACT_CATALOG_BYTES: usize = discovery_contract::MAX_COMPACT_CATALOG_BYTES;
pub(crate) const LIST_EXTENSIONS_TOOL_NAME: &str = discovery_contract::DISCOVERY_TOOL_NAMES[0];
pub(crate) const INSPECT_EXTENSIONS_TOOL_NAME: &str = discovery_contract::DISCOVERY_TOOL_NAMES[1];
pub use discovery_preferences::DiscoveryPreferences;
pub use public_api::{
    discovery_preferences, invoke_ui_action, report_ui_mount_failure, set_discovery_preferences,
    ui_catalog,
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
#[cfg(feature = "e2e")]
pub(crate) use startup::initialize;
pub use startup::initialize_on_startup;
pub(crate) use tool_bridge::definitions as extension_tool_definitions;
pub(crate) use tool_bridge::{core_fallback, without_core_fallback};
pub use tool_bridge::{merge_definitions as merge_tool_definitions, validate_arguments};
pub(crate) use tool_result::unavailable as unavailable_tool_result;

pub(crate) use public_api::{
    close_command_error, record_tool_invocation, revoke_extension, MAX_DISCOVERED_PLUGINS,
    MAX_EXTENSION_TOOLS, MAX_PERMISSION_SUMMARY_CHARS,
};

pub(crate) use extension_internal_exports::*;
pub(crate) use installer::{
    install_git as install_git_source, install_npm as install_npm_source,
    uninstall as uninstall_extension, update as update_managed_extension,
};
pub(crate) use manifest::load_local as install_local;
pub(crate) use operation_error::{report as report_operation_error, Operation};
pub(crate) use operation_failure::OperationFailure;
pub(crate) use resource_identifier::parse as parse_qualified_contribution_id;
pub(crate) use resource_loader::{
    load_skill_for_session as load_extension_skill_for_session, LoadedResource, ResourceLoadError,
};
