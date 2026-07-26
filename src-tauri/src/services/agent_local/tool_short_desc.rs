//! One-line descriptions for optional tools, used by the disabled-tools hint
//! injected into the system prompt. Locked (always-on) tools are NOT described
//! here because they are never part of the hint.
//!
//! Keep each description under 100 chars. Plain English, no jargon.

/// Returns the short description for an optional tool id, or `None` for
/// locked tools and unknown ids.
pub fn tool_short_desc(id: &str) -> Option<&'static str> {
    let desc = match id {
        // workflow (default-on)
        "load_skill" => "Load a skill by its source-qualified ID",
        "ask_user_choice" => "Ask the user to choose between options",
        "delegate_task" => "Spawn a subagent for parallel or isolated work",
        "list_subagents" => "List child subagents for this session",
        "get_subagent" => "Inspect one child subagent",
        "cancel_subagent" => "Cancel a running child subagent",
        "message_subagent" => "Send a new instruction to a child subagent",
        "archive_subagent" => "Archive a completed child subagent",
        "inspect_subagent_changes" => "Review a coder subagent's isolated changes",
        "apply_subagent_changes" => "Apply approved coder changes to the parent project",
        "discard_subagent_changes" => "Discard rejected coder changes",
        "planmode" => "Publish a plan and request user approval (Plan mode)",
        "exitplanmode" => "Exit Plan mode after the plan is approved",
        // todo_list
        "todo_write" => "Create or update a task checklist",
        "todo_history" => "List saved todo checklists for this session",
        "todo_pause" => "Pause the active checklist",
        "todo_resume" => "Resume a saved checklist by id",
        "todo_delete" => "Delete a checklist",
        "agent_diagnostics" => "Read recent safe stream diagnostics",
        // git_branches
        "create_branch" => "Create a new git branch from HEAD",
        "checkout_branch" => "Switch to an existing git branch",
        // forecast
        "forecast" => "Run a time-series forecast from data or a file",
        "forecast_models" => "List available forecast models",
        "forecast_analyze" => "Add annotations or scenarios to a saved forecast",
        "forecast_read" => "Read saved forecast analyses",
        "forecast_data_audit" => "Audit Forecast data quality",
        "forecast_backtest" => "Evaluate models with rolling temporal backtests",
        "forecast_compare_models" => "Compare saved Forecast backtest results",
        // office: spreadsheet
        "read_spreadsheet" => "Read data from an Excel or CSV file",
        "write_spreadsheet" => "Create or modify an Excel file",
        // office: document
        "read_document" => "Extract text from PDF or Word files",
        "write_document" => "Create a Word .docx document",
        // office: images
        "read_image" => "Read image metadata (dimensions, format)",
        "process_image" => "Resize, crop, or convert an image",
        // locked tools and unknown ids: no hint
        _ => return None,
    };
    Some(desc)
}
