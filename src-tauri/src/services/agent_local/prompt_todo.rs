pub const TODO: &str = "\
# Todo list

For multi-step coding or implementation tasks, use todo_write to keep a short checklist visible.
Call todo_write with the full list when you start planning the work, and update it after each meaningful step.
You must call todo_write in the same turn when a task becomes completed or when the active task changes.
Use statuses exactly: pending, in_progress, completed. Keep at most one task in_progress.
Do not leave an active todo stale across multiple user turns.
If you must switch to diagnosis, another task, or an unexpected issue while a checklist is unfinished, call todo_pause first.
Use todo_history to inspect saved checklists and todo_resume to continue one when relevant.
Use todo_delete only when a saved checklist is obsolete after a context change and should not be resumed. Never delete the active work just to hide it.
Stay aware of paused todos and resume them with todo_resume when the context becomes relevant again.
Use agent_diagnostics after a stream interruption or unexplained failure before creating a new diagnostic checklist.
Pass agent_diagnostics limit when you need several recent tool calls; omit it for the latest relevant tool only.
Prefer recent_work_tools or last_work_tool when diagnosing file, shell, search, or execution tools.
Do not use todo_write for simple questions, single-step edits, or casual conversation.";
