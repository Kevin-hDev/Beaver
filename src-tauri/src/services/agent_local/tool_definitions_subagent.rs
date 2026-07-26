use serde_json::Value;

pub fn delegate_task_definition() -> Value {
    super::tool_definitions::tool_def(
        "delegate_task",
        "Spawn an autonomous subagent to handle a subtask in isolation. Results come back to you; they are NOT shown to the user, so you must relay a summary. \
         Types: 'explorer' (read-only: read_file, list_dir, grep, glob, web_search, web_fetch) for research, file investigation, web lookups; \
         or 'coder' (file creation/modification in an isolated git worktree) for parallel implementation work. \
         The subagent keeps working in its visible child session while you continue. The parent stream stays active until subagents from the current turn finish. \
         When NOT to use: reading a specific known file — use read_file directly; searching for a single class/function — use grep or glob directly; a 1-2 step task — do it yourself. \
         IMPORTANT: once you delegate a task, do NOT do the same work yourself and do NOT write a final answer before the required subagent reports arrive. \
         Write a structured prompt using XML tags: <context>, <task>, <constraints>, <output_format>. Terse prompts produce shallow results. \
         You can spawn multiple subagents in parallel for independent subtasks.",
        serde_json::json!({
            "type": "object",
            "properties": {
                "prompt": {
                    "type": "string",
                    "description": "Structured instruction for the subagent. Use XML tags: \
                     <context> (background info the subagent needs), \
                     <task> (what to do — be specific), \
                     <constraints> (boundaries, what NOT to do), \
                     <output_format> (expected response structure). \
                     A well-structured prompt produces significantly better results."
                },
                "subagent_type": {
                    "type": "string",
                    "enum": ["explorer", "coder"],
                    "description": "explorer = read-only research, coder = file modifications in isolated worktree"
                },
                "name": {
                    "type": "string",
                    "description": "Legacy mission label. The visible agent identity is fixed by type."
                },
                "display_name": {
                    "type": "string",
                    "description": "Legacy mission label. Visible identities are Claudiator for coder and Geminitor for explorer."
                },
                "description": {
                    "type": "string",
                    "description": "Short visible mission description for the child session."
                },
                "subagent_id": {
                    "type": "string",
                    "description": "Existing child session id to reuse instead of creating a fresh subagent."
                }
            },
            "required": ["prompt", "subagent_type"]
        }),
    )
}

pub fn subagent_control_definitions() -> Vec<Value> {
    use super::tool_definitions::tool_def;
    vec![
        tool_def(
            "list_subagents",
            "List subagents owned by this parent session with short status, name, description, and id.",
            serde_json::json!({ "type": "object", "properties": {} }),
        ),
        tool_def(
            "get_subagent",
            "Get status, description, and final summary for one child subagent session.",
            serde_json::json!({
                "type": "object",
                "properties": { "subagent_id": { "type": "string", "description": "Child subagent session id" } },
                "required": ["subagent_id"]
            }),
        ),
        tool_def(
            "cancel_subagent",
            "Cancel a running child subagent owned by this parent session.",
            serde_json::json!({
                "type": "object",
                "properties": { "subagent_id": { "type": "string", "description": "Child subagent session id" } },
                "required": ["subagent_id"]
            }),
        ),
        tool_def(
            "message_subagent",
            "Send a new instruction to an existing child subagent session. Running sessions queue the instruction for a later run.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "subagent_id": { "type": "string", "description": "Child subagent session id" },
                    "prompt": { "type": "string", "description": "Instruction to send to the child subagent" }
                },
                "required": ["subagent_id", "prompt"]
            }),
        ),
        tool_def(
            "archive_subagent",
            "Archive a completed child subagent session owned by this parent. Running subagents must be cancelled first.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "subagent_id": { "type": "string", "description": "Child subagent session id" }
                },
                "required": ["subagent_id"]
            }),
        ),
    ]
}

pub fn subagent_change_definitions() -> Vec<Value> {
    use super::tool_definitions::tool_def;
    [
        ("inspect_subagent_changes", "Inspect a coder subagent change before deciding what to do."),
        ("apply_subagent_changes", "Apply a coder subagent change to the current parent branch."),
        (
            "discard_subagent_changes",
            "Discard a rejected or manually reproduced coder change and delete its temporary branch.",
        ),
    ]
    .into_iter()
    .map(|(name, description)| {
        tool_def(
            name,
            description,
            serde_json::json!({
                "type": "object",
                "properties": {
                    "subagent_id": { "type": "string" },
                    "change_id": { "type": "string" }
                },
                "required": ["subagent_id", "change_id"]
            }),
        )
    })
    .collect()
}
