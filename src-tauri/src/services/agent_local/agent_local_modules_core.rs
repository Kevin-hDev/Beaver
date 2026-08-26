pub mod agent_definition;
pub mod agent_loop;
pub mod agent_loop_completion;
mod agent_loop_compression;
pub mod agent_loop_errors;
pub mod agent_loop_finish;
mod agent_loop_interrupted;
pub mod agent_loop_limits;
mod agent_loop_ollama_request;
mod agent_loop_tool_batch;
pub mod agent_loop_plan;
pub mod agent_loop_support;
pub mod agent_loop_thinking_retry;
pub mod agent_md;
pub mod agent_resource_access;
pub mod agent_settings;
pub mod agent_work_supervision;
#[cfg(test)]
mod agent_work_supervision_tests;
pub mod app_handle_global;
pub mod chat_message;
#[cfg(test)]
mod chat_message_tests;
pub mod circuit_breaker;
#[cfg(test)]
pub mod circuit_breaker_tests;
pub mod clone_git;
pub mod clone_git_checks;
pub mod clone_git_cleanup;
pub mod clone_git_link;
pub mod clone_roots;
pub mod clone_session;
pub mod clone_session_build;
pub mod clone_summary;
pub mod clone_summary_ops;
pub mod clone_summary_prompt;
pub mod compress_hook;
pub mod diagnostic_args;
#[cfg(test)]
mod diagnostic_args_tests;
pub mod diagnostic_redaction;
pub mod directory_access;
mod directory_access_scope;
mod directory_policy;
pub mod eager_dispatch;
pub mod extension_discovery_prompt;
mod extension_session_plugins;
pub mod extension_session_state;
pub mod extension_tool_mask;
pub mod extension_tool_selection;
mod extension_tool_set_apply;
mod extension_tool_set_diagnostics;
pub mod interactive_choice_gate;
pub mod memory_archive;
pub mod memory_context;
pub mod memory_context_usage;
pub mod memory_format;
pub mod memory_format_update;
pub mod memory_index;
pub mod memory_io;
pub mod memory_overview;
pub mod memory_path_classification;
pub mod memory_path_security;
pub mod memory_paths;
pub mod memory_project_id;
pub mod memory_project_labels;
pub mod memory_project_migration;
pub mod memory_prompt;
pub mod memory_runtime;
pub mod memory_settings;
pub mod memory_store;
pub mod memory_tool;
mod memory_tool_error;
pub mod memory_types;
pub mod model_customizations;
#[cfg(test)]
mod model_customizations_tests;
pub mod model_size;
pub mod modelfile_parser;
pub mod ollama_client;
#[cfg(test)]
mod ollama_client_integration_tests;
pub mod ollama_collect;
pub mod ollama_model_helpers;
pub mod ollama_modelfile_create;
pub mod ollama_modelfile_parameters;
#[cfg(test)]
mod ollama_modelfile_parameters_tests;
pub mod ollama_native_prompts;
#[cfg(test)]
mod ollama_native_prompts_tests;
pub mod ollama_parameter_summary;
#[cfg(test)]
mod ollama_parameter_summary_tests;
pub mod ollama_parameter_validation;
pub mod ollama_registry;
pub mod ollama_registry_details;
#[cfg(test)]
mod ollama_registry_tests;
pub mod ollama_retry_indicator;
pub mod ollama_runtime;
pub mod ollama_stream;
mod ollama_stream_filter;
pub mod ollama_stream_process;
#[cfg(test)]
mod ollama_stream_process_tests;
pub mod ollama_stream_request;
pub mod ollama_stream_retry;
mod ollama_stream_policy;
pub mod ollama_thinking_retry;
pub mod ollama_tool_parse_retry;
pub mod ollama_tool_role;
#[cfg(test)]
mod ollama_tool_role_tests;
pub mod ollama_wire;
pub(crate) mod parent_message_inbox;
mod permission_allow_cache;
pub mod permission_bash;
pub mod permission_gate;
#[cfg(test)]
pub mod permission_gate_tests;
pub mod permission_policy;
pub mod plan_mode_controller;
pub mod plan_mode_debug;
pub mod private_data_access;
pub mod project_store;
pub mod provider_tool_limits;
pub mod security;
pub mod sensitive_data;
mod shell_sandbox_diagnostics;
pub mod subagent_activity;
pub mod subagent_completion;
mod subagent_completion_boundary;
#[cfg(test)]
mod subagent_completion_boundary_tests;
#[cfg(test)]
mod subagent_completion_capacity_tests;
#[cfg(test)]
mod subagent_completion_tests;
#[cfg(test)]
mod subagent_correction_capacity_tests;
#[cfg(test)]
mod subagent_delegate_prompt_tests;
#[cfg(test)]
mod subagent_event_completion_failure_tests;
#[cfg(test)]
mod subagent_event_completion_signal_tests;
#[cfg(test)]
#[path = "subagent_terminal_wait_tests.rs"]
mod subagent_event_terminal_tests;
#[cfg(test)]
mod subagent_event_wait_tests;
#[cfg(test)]
mod subagent_execution_ownership_tests;
#[cfg(test)]
mod subagent_failure_queue_tests;
pub mod subagent_hidden_reports;
pub(crate) mod subagent_instruction_delivery;
#[cfg(test)]
mod subagent_instruction_delivery_tests;
#[cfg(test)]
mod subagent_instruction_execution_race_tests;
#[cfg(test)]
mod subagent_instruction_limit_tests;
#[cfg(test)]
mod subagent_instruction_wiring_tests;
pub mod subagent_live_state;
pub mod subagent_orchestration;
pub mod subagent_orchestration_context;
#[cfg(test)]
mod subagent_orchestration_race_tests;
pub mod subagent_panic_supervisor;
#[cfg(test)]
mod subagent_panic_supervisor_tests;
pub mod subagent_parent_guidance;
#[cfg(test)]
mod subagent_parent_guidance_tests;
pub mod subagent_profile;
pub mod subagent_prompts;
#[cfg(test)]
pub mod subagent_prompts_tests;
#[cfg(test)]
mod subagent_redeploy_atomic_tests;
pub mod subagent_registry;
#[cfg(test)]
mod subagent_registry_test_support;
#[cfg(test)]
pub mod subagent_registry_tests;
#[cfg(test)]
mod subagent_report_ack_tests;
mod subagent_report_context;
mod subagent_report_delivery;
#[cfg(test)]
mod subagent_report_delivery_tests;
#[cfg(test)]
mod subagent_review_fail_closed_tests;
#[cfg(test)]
mod subagent_same_run_tests;
pub mod subagent_spawn_channel;
#[cfg(test)]
mod subagent_spawn_channel_tests;
pub mod subagent_startup_cleanup;
pub mod subagent_status;
pub mod subagent_summary;
pub mod subagent_task;
mod subagent_task_spawn;
pub mod subagent_task_stream;
#[cfg(test)]
pub mod subagent_task_tests;
#[cfg(test)]
mod subagent_terminal_consumption_tests;
mod subagent_terminal_signal;
#[cfg(test)]
mod subagent_terminal_wait_test_support;
pub mod subagent_tool_control;
#[cfg(test)]
mod subagent_tool_control_tests;
#[cfg(test)]
mod subagent_worktree_ownership_tests;
#[cfg(test)]
mod subagent_worktree_wiring_tests;
pub mod system_prompt_resolver;
#[cfg(test)]
mod system_prompt_settings_tests;
pub mod system_prompt_store;
pub mod system_prompt_types;
