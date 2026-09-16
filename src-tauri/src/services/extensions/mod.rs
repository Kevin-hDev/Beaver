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
include!("extensions_modules_core.rs");
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
include!("extensions_modules_events.rs");
mod extension_internal_exports;
pub(crate) mod extension_recovery;
mod fingerprint;
mod fingerprint_paths;
mod git_checkout;
mod git_package;
mod git_reference;
mod git_resolution;
mod git_resolution_history;
mod git_source;
mod git_transport;
mod host_channel;
mod host_core_call;
mod host_identity;
mod host_load_tracker;
mod host_paths;
mod host_process;
mod host_reader;
mod host_reader_line;
mod host_reader_scope;
#[cfg(test)]
mod host_reader_scope_tests;
mod host_stop_boundary;
#[cfg(test)]
mod host_stop_boundary_tests;
pub(crate) mod install_jobs;
mod install_retry;
mod install_signal;
mod installer;
mod installer_process;
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
mod npm_paths;
mod npm_runner;
mod npm_source;
mod npm_workspace;
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
mod registry_startup;
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
include!("extensions_modules_runtime.rs");
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
mod runtime_status;
mod runtime_sync;
mod runtime_sync_apply;
#[cfg(test)]
mod runtime_sync_apply_tests;
mod runtime_sync_contributions;
mod runtime_sync_interceptors;
mod runtime_ui_diagnostics;
mod runtime_version;
mod source_validation;
mod startup;
mod storage;
mod storage_format;
mod storage_migration;
mod tool_bridge;
mod tool_interception;
mod tool_interception_catalog;
mod tool_interception_failure;
mod tool_interception_result;
#[cfg(test)]
mod tool_interception_tests;
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
mod ui_builder_build;
mod ui_builder_paths;
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
include!("extensions_modules_work.rs");
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
include!("test_modules.inc.rs");

include!("api_exports.inc.rs");
#[cfg(test)]
mod storage_resilience_tests;
