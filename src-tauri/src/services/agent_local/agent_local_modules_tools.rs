pub mod subagent_coder_project;
pub mod subagent_directory_workspace;
pub mod subagent_explorer_process;
pub mod subagent_working_dir;
pub mod subagent_worktree;
include!("agent_local_modules_shell.rs");
pub mod extension_tool_set;
pub mod extension_skill_loader;
#[cfg(test)]
mod extension_skill_loader_tests;
#[cfg(debug_assertions)]
pub mod fixture_tool_executor;
pub mod tool_automation;
#[cfg(test)]
mod tool_artifact_tests;
#[cfg(test)]
mod tool_automation_tests;
mod tool_automation_validation;
pub mod tool_availability;
pub mod tool_catalog;
mod tool_catalog_filter;
#[cfg(test)]
pub mod tool_catalog_tests;
pub mod tool_definitions;
pub mod tool_definitions_automation;
pub mod tool_definitions_chat;
pub mod tool_definitions_core;
pub mod tool_definitions_extensions;
pub mod tool_definitions_forecast;
pub mod tool_definitions_git;
pub mod tool_definitions_interactive;
pub mod tool_definitions_mcp;
pub mod tool_definitions_office;
pub mod tool_definitions_plan;
pub mod tool_definitions_search;
pub mod tool_definitions_skills;
pub mod tool_definitions_subagent;
pub mod tool_definitions_todo;
pub mod tool_definitions_web;
pub mod tool_delegate;
pub mod tool_delegate_child;
mod tool_delegate_identity;
mod tool_delegate_prompt;
mod tool_dispatch_trace;
pub mod tool_dispatcher;
pub mod tool_dispatcher_delegate;
mod tool_dispatcher_entry;
mod tool_dispatcher_error;
pub mod tool_dispatcher_fallback;
mod tool_dispatcher_finalize;
pub mod tool_dispatcher_forecast;
pub mod tool_dispatcher_forecast_analyze;
mod tool_dispatcher_forecast_annotation;
pub mod tool_dispatcher_forecast_data_audit;
pub mod tool_dispatcher_forecast_evaluation;
pub mod tool_dispatcher_forecast_execute;
mod tool_dispatcher_forecast_load;
pub mod tool_dispatcher_forecast_models;
mod tool_dispatcher_forecast_models_support;
pub mod tool_dispatcher_forecast_output;
pub mod tool_dispatcher_forecast_persist;
pub mod tool_dispatcher_forecast_run;
pub mod tool_dispatcher_forecast_runtime;
pub mod tool_dispatcher_forecast_scenario_params;
pub mod tool_dispatcher_forecast_selection;
pub mod tool_dispatcher_mcp;
pub mod tool_dispatcher_office;
mod tool_dispatcher_route;
pub mod tool_dispatcher_shell;
mod tool_dispatcher_shell_error;
#[cfg(test)]
pub mod tool_dispatcher_tests;
#[cfg(test)]
pub mod tool_document_format_tests;
pub mod tool_document_read;
#[cfg(test)]
pub mod tool_document_read_tests;
pub mod tool_document_write;
pub mod tool_document_write_list;
pub mod tool_document_write_numbering;
mod tool_document_write_run;
pub mod tool_document_write_styles;
mod tool_document_write_table;
#[cfg(test)]
pub mod tool_document_write_tests;
pub mod tool_document_write_xml;
pub mod tool_execution_outcome;
mod tool_execution_artifacts;
pub mod tool_executor;
pub mod tool_executor_compression;
pub mod tool_executor_delegate_batch;
mod tool_executor_delegate_launch;
pub mod tool_executor_diagnostics;
mod tool_executor_errors;
pub mod tool_executor_helpers;
pub mod tool_executor_parallel;
pub mod tool_executor_parallel_batch;
pub mod tool_executor_parallel_dispatch;
mod tool_executor_parallel_finalize;
#[cfg(test)]
pub mod tool_executor_parallel_tests;
pub mod tool_executor_parallel_write;
pub mod tool_executor_plan;
pub mod tool_executor_read_only;
pub mod tool_executor_results;
pub mod tool_executor_sequential;
mod tool_executor_sequential_support;
pub mod tool_executor_write;
pub mod tool_extension_list;
pub mod tool_extension_inspect;
pub mod tool_extension_resource;
#[cfg(test)]
mod tool_extension_resource_tests;
mod tool_extension_catalog_diagnostics;
#[allow(dead_code)]
mod extension_discovery_contract {
    include!(concat!(env!("OUT_DIR"), "/extension_discovery_contract.rs"));
}
#[cfg(test)]
mod extension_discovery_contract_tests;
pub mod tool_file_changes;
mod tool_file_error;
mod tool_file_write;
pub mod tool_files;
#[cfg(test)]
pub mod tool_files_tests;
mod tool_git_error;
pub mod tool_glob;
pub mod tool_grep;
pub mod tool_group_catalog;
pub mod tool_hooks;
#[cfg(test)]
pub mod tool_hooks_tests;
mod tool_image_inspect;
pub mod tool_image_process;
#[cfg(test)]
mod tool_image_process_contract_tests;
mod tool_image_process_geometry;
#[cfg(test)]
pub mod tool_image_process_limits_tests;
#[cfg(test)]
pub mod tool_image_process_tests;
pub mod tool_interactive;
pub mod tool_interactive_parse;
#[cfg(test)]
pub mod tool_interactive_recommendation_tests;
#[cfg(test)]
pub mod tool_interactive_tests;
pub mod tool_list_dir;
#[cfg(test)]
mod tool_list_dir_tests;
pub mod tool_mcp;
pub mod tool_pending_artifact_batch;
mod tool_pending_artifact_errors;
mod tool_pending_artifact_inspect;
mod tool_pending_artifact_read;
mod tool_pending_artifact_revalidate;
pub mod tool_pending_artifacts;
mod tool_mcp_call;
mod tool_office_array;
#[cfg(test)]
mod tool_office_array_tests;
pub mod tool_office_limits;
pub mod tool_office_utils;
pub mod tool_plan;
pub mod tool_plan_approval;
pub mod tool_plan_approval_request;
pub mod tool_plan_guard;
pub mod tool_plan_messages;
pub mod tool_plan_storage;
pub mod tool_prompt_filter;
pub mod tool_result_budget;
#[cfg(test)]
pub mod tool_result_budget_tests;
pub mod tool_result_contract;
pub mod tool_result_model;
pub(crate) mod tool_result_model_compact;
pub mod tool_result_truncate;
pub mod tool_scan_timeout;
#[cfg(test)]
mod tool_search_result_tests;
pub mod tool_skill_loader;
mod tool_spreadsheet_border;
pub mod tool_spreadsheet_calamine;
mod tool_spreadsheet_error;
#[cfg(test)]
pub mod tool_spreadsheet_format_tests;
mod tool_spreadsheet_range;
pub mod tool_spreadsheet_read;
#[cfg(test)]
pub mod tool_spreadsheet_read_tests;
pub mod tool_spreadsheet_write;
pub mod tool_spreadsheet_write_edit;
mod tool_spreadsheet_write_format;
pub mod tool_spreadsheet_write_new;
mod tool_spreadsheet_write_new_format;
#[cfg(test)]
pub mod tool_spreadsheet_write_tests;
pub mod tool_subagent_control;
pub mod tool_subagent_format;
mod tool_subagent_message;
pub mod tool_todo;
mod tool_todo_delete;
#[cfg(test)]
mod tool_todo_memory_tests;
pub mod tool_todo_neglect;
pub mod tool_todo_parse;
pub mod tool_todo_state;
pub mod tool_todo_summary;
pub mod tool_validate;
mod tool_web_error;
pub mod tool_web_fetch;
pub mod tool_web_fetch_ip;
#[cfg(test)]
pub mod tool_web_fetch_network_tests;
#[cfg(test)]
pub mod tool_web_fetch_tests;
pub mod tool_web_search;
mod tool_workspace_notice;
pub mod translation_cache;
pub mod translator;
include!("agent_local_modules_tool_types.rs");
pub mod write_guard;
pub mod write_guard_extract;
#[cfg(test)]
pub mod write_guard_helpers_tests;
pub mod write_guard_registry;
#[cfg(test)]
pub mod write_guard_tests;
