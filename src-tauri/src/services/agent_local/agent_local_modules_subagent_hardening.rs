pub mod subagent_archive;
#[cfg(test)]
mod backend_stream_generation_tests;
#[cfg(test)]
mod session_store_update_race_tests;
#[cfg(test)]
mod subagent_archive_tests;
pub mod subagent_cancellation;
#[cfg(test)]
mod subagent_cancellation_tests;
#[cfg(test)]
mod subagent_parent_stream_ownership_tests;
#[cfg(test)]
mod subagent_worktree_cleanup_tests;
#[cfg(test)]
mod subagent_worktree_inspection_tests;
#[cfg(test)]
mod subagent_worktree_owner_validation_tests;
#[cfg(test)]
mod subagent_spawn_event_order_tests;
mod subagent_completion_events;
mod subagent_completion_ownership;
#[cfg(test)]
mod subagent_empty_tool_history_tests;
mod subagent_history;
#[cfg(test)]
mod subagent_history_tests;
#[cfg(test)]
mod subagent_history_failure_tests;
mod subagent_worktree_cleanup;
mod subagent_worktree_identity;
#[cfg(test)]
mod subagent_max_turn_tests;
mod subagent_turn_limit;
mod subagent_tool_guard;
pub(crate) mod session_permission_state;
pub(crate) mod agent_send_preflight;
#[cfg(test)]
mod agent_send_preflight_tests;
#[cfg(test)]
mod session_permission_state_tests;
pub(crate) mod subagent_tool_profile;
mod subagent_explorer_bash;
mod subagent_explorer_bash_options;
#[cfg(test)]
mod subagent_explorer_bash_flags_tests;
mod subagent_prompt_sections;
mod subagent_runtime_context;
mod subagent_change_store;
#[cfg(test)]
mod subagent_change_permission_tests;
#[cfg(test)]
mod subagent_change_store_tests;
mod subagent_git_actions;
mod subagent_git_command;
mod subagent_git_lock;
#[cfg(test)]
mod subagent_git_lock_tests;
mod subagent_git_run;
mod subagent_directory_change;
mod subagent_directory_apply;
mod subagent_directory_replay;
mod subagent_directory_limits;
mod subagent_directory_transaction;
mod subagent_task_change;
mod subagent_task_failure;
mod types_subagent_change;
mod tool_subagent_changes;
#[cfg(test)]
mod subagent_git_lifecycle_tests;
#[cfg(test)]
mod subagent_git_dirty_parent_tests;
#[cfg(test)]
mod subagent_git_hook_tests;
#[cfg(test)]
mod subagent_directory_workspace_tests;
#[cfg(test)]
mod subagent_directory_conflict_tests;
#[cfg(test)]
mod subagent_directory_recovery_tests;
mod tool_delegate_spawned;
#[cfg(test)]
mod subagent_tool_profile_tests;
#[cfg(test)]
mod subagent_tool_guard_tests;
#[cfg(test)]
mod subagent_tool_runtime_tests;
#[cfg(test)]
mod subagent_inheritance_tests;
#[cfg(test)]
mod subagent_terminal_event_order_tests;
#[cfg(test)]
mod subagent_terminal_event_consistency_tests;
