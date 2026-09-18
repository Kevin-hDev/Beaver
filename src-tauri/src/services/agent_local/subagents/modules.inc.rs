pub(crate) mod parent_message_inbox;
pub mod subagent_activity;
pub mod subagent_archive;
#[cfg(test)]
mod subagent_archive_tests;
pub mod subagent_cancellation;
#[cfg(test)]
mod subagent_cancellation_tests;
#[cfg(test)]
mod subagent_change_permission_tests;
mod subagent_change_store;
#[cfg(test)]
mod subagent_change_store_tests;
pub mod subagent_coder_project;
pub mod subagent_completion;
mod subagent_completion_boundary;
#[cfg(test)]
mod subagent_completion_boundary_tests;
#[cfg(test)]
mod subagent_completion_capacity_tests;
mod subagent_completion_events;
mod subagent_completion_ownership;
#[cfg(test)]
mod subagent_completion_tests;
#[cfg(test)]
mod subagent_correction_capacity_tests;
#[cfg(test)]
mod subagent_delegate_prompt_tests;
mod subagent_directory_apply;
mod subagent_directory_change;
#[cfg(test)]
mod subagent_directory_conflict_tests;
mod subagent_directory_git;
mod subagent_directory_limits;
#[cfg(test)]
mod subagent_directory_recovery_tests;
mod subagent_directory_replay;
mod subagent_directory_transaction;
pub mod subagent_directory_workspace;
#[cfg(test)]
mod subagent_directory_workspace_tests;
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
mod subagent_explorer_bash;
#[cfg(test)]
mod subagent_explorer_bash_flags_tests;
mod subagent_explorer_bash_options;
pub mod subagent_explorer_process;
pub(crate) mod subagent_extension_api;
#[cfg(test)]
mod subagent_failure_queue_tests;
mod subagent_git_actions;
mod subagent_git_command;
#[cfg(test)]
mod subagent_git_dirty_parent_tests;
#[cfg(test)]
mod subagent_git_hook_tests;
#[cfg(test)]
mod subagent_git_lifecycle_tests;
mod subagent_git_lock;
#[cfg(test)]
mod subagent_git_lock_tests;
mod subagent_git_run;
pub mod subagent_hidden_reports;
#[cfg(test)]
mod subagent_inheritance_tests;
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
#[cfg(test)]
mod subagent_parent_stream_ownership_tests;
pub mod subagent_profile;
mod subagent_prompt_sections;
pub mod subagent_prompts;
#[cfg(test)]
pub mod subagent_prompts_tests;
#[cfg(test)]
mod subagent_redeploy_atomic_tests;
#[cfg(test)]
mod subagent_redeployment_tests;
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
pub(crate) mod subagent_report_overflow;
#[cfg(test)]
mod subagent_review_fail_closed_tests;
mod subagent_runtime_context;
#[cfg(test)]
mod subagent_same_run_tests;
pub mod subagent_spawn_channel;
#[cfg(test)]
mod subagent_spawn_channel_tests;
#[cfg(test)]
mod subagent_spawn_event_order_tests;
pub mod subagent_startup_cleanup;
pub mod subagent_status;
pub mod subagent_summary;
pub mod subagent_task;
mod subagent_task_change;
mod subagent_task_failure;
mod subagent_task_spawn;
pub mod subagent_task_stream;
#[cfg(test)]
pub mod subagent_task_tests;
#[cfg(test)]
mod subagent_terminal_consumption_tests;
#[cfg(test)]
mod subagent_terminal_event_consistency_tests;
#[cfg(test)]
mod subagent_terminal_event_order_tests;
mod subagent_terminal_signal;
#[cfg(test)]
mod subagent_terminal_wait_test_support;
pub mod subagent_tool_control;
#[cfg(test)]
mod subagent_tool_control_tests;
mod subagent_tool_guard;
#[cfg(test)]
mod subagent_tool_guard_tests;
pub(crate) mod subagent_tool_profile;
#[cfg(test)]
mod subagent_tool_profile_tests;
#[cfg(test)]
mod subagent_tool_runtime_tests;
pub mod subagent_working_dir;
pub mod subagent_worktree;
mod subagent_worktree_cleanup;
#[cfg(test)]
mod subagent_worktree_cleanup_tests;
mod subagent_worktree_identity;
#[cfg(test)]
mod subagent_worktree_inspection_tests;
#[cfg(test)]
mod subagent_worktree_owner_validation_tests;
#[cfg(test)]
mod subagent_worktree_ownership_tests;
#[cfg(test)]
mod subagent_worktree_wiring_tests;
mod types_subagent_change;
