use crate::services::agent_local::tool_result_contract::ToolErrorCategory;
use crate::services::agent_local::types_tools::ToolResult;
use crate::services::agent_local::{
    tool_files, tool_glob, tool_grep, tool_web_fetch, tool_web_search,
};
use serde_json::Value;
use std::path::Path;

#[cfg(test)]
pub use super::tool_dispatcher_entry::dispatch;
pub(crate) use super::tool_dispatcher_entry::dispatch_for_mode;
pub(crate) use super::tool_dispatcher_entry::dispatch_with_progress;
#[cfg(test)]
pub(crate) use super::tool_dispatcher_error::enrich as enrich_error;
pub use crate::services::agent_local::tool_definitions::get_tool_definitions;
pub use crate::services::agent_local::tool_definitions_chat::get_chat_tool_definitions;

pub(super) async fn dispatch_inner(
    tool_name: &str,
    args: &Value,
    working_dir: &Path,
    trace: super::tool_dispatch_trace::DispatchTrace<'_>,
    cancel: tokio_util::sync::CancellationToken,
    profile: Option<super::subagent_tool_profile::SubagentToolProfile>,
    progress: Option<super::tool_bash_progress::ShellProgress>,
) -> ToolResult {
    let session_id = trace.session_id;
    if super::extension_tool_set::native_only_for_session(session_id)
        && matches!(
            tool_name,
            crate::services::extensions::LIST_EXTENSIONS_TOOL_NAME
                | crate::services::extensions::INSPECT_EXTENSIONS_TOOL_NAME
        )
    {
        return ToolResult::unavailable(
            crate::services::extensions::error_codes::STATE_UNAVAILABLE,
            "Extensions indisponibles.",
            true,
        );
    }
    match tool_name {
        "bash" | "bash_control" => {
            super::tool_dispatcher_shell::dispatch(
                tool_name,
                args,
                working_dir,
                session_id,
                cancel,
                profile,
                progress,
            )
            .await
        }
        "read_file" => {
            let path = args["path"].as_str().unwrap_or("");
            let offset = args["offset"].as_u64().unwrap_or(0) as usize;
            let limit = args["limit"]
                .as_u64()
                .unwrap_or(tool_files::DEFAULT_LIMIT as u64) as usize;
            tool_files::read_file(path, working_dir, offset, limit).await
        }
        "write_file" => {
            let path = args["path"].as_str().unwrap_or("");
            let content = args["content"].as_str().unwrap_or("");
            tool_files::write_file(path, content, working_dir).await
        }
        "edit_file" => {
            let path = args["path"].as_str().unwrap_or("");
            let old = args["old_string"].as_str().unwrap_or("");
            let new = args["new_string"].as_str().unwrap_or("");
            tool_files::edit_file(path, old, new, working_dir).await
        }
        "list_dir" => {
            let path = args["path"].as_str().unwrap_or(".");
            tool_files::list_dir(path, working_dir).await
        }
        "grep" => {
            let pattern = args["pattern"].as_str().unwrap_or("");
            let path = args["path"].as_str();
            let glob_filter = args["glob"].as_str();
            tool_grep::grep(pattern, path, glob_filter, working_dir).await
        }
        "glob" => {
            let pattern = args["pattern"].as_str().unwrap_or("");
            let path = args["path"].as_str();
            tool_glob::glob_files(pattern, path, working_dir).await
        }
        "web_search" => {
            let query = args["query"].as_str().unwrap_or("");
            match tool_web_search::web_search(query).await {
                Ok(results) => {
                    let text = results
                        .iter()
                        .map(|r| format!("**{}**\n{}\n{}", r.title, r.url, r.snippet))
                        .collect::<Vec<_>>()
                        .join("\n\n");
                    ToolResult::ok(text)
                }
                Err(error) => super::tool_web_error::search(error),
            }
        }
        "web_fetch" => {
            let url = args["url"].as_str().unwrap_or("");
            match tool_web_fetch::fetch_url(url).await {
                Ok(content) => ToolResult::ok(content),
                Err(error) => super::tool_web_error::fetch(error),
            }
        }
        name if name == crate::services::extensions::LIST_EXTENSIONS_TOOL_NAME => {
            super::tool_extension_list::execute().await
        }
        name if name == crate::services::extensions::INSPECT_EXTENSIONS_TOOL_NAME => {
            super::tool_extension_inspect::execute(args, session_id, trace.request_id).await
        }
        "todo_write" => super::tool_todo::execute(args, session_id).await,
        "todo_history" => super::tool_todo::execute_history(args, session_id).await,
        "todo_pause" => super::tool_todo::execute_pause(args, session_id).await,
        "todo_resume" => super::tool_todo::execute_resume(args, session_id).await,
        "todo_delete" => super::tool_todo::execute_delete(args, session_id).await,
        "ask_user_choice" => ToolResult::error(
            "Contexte interactif indisponible.",
            "interactive_context_unavailable",
            ToolErrorCategory::Unavailable,
            false,
        ),
        "plan_mode" => ToolResult::error(
            "Contexte plan indisponible.",
            "plan_context_unavailable",
            ToolErrorCategory::Unavailable,
            false,
        ),
        "load_skill" => {
            let skill_id = args["skill_id"].as_str().unwrap_or("");
            match super::extension_skill_loader::load_skill_for_session(skill_id, session_id).await
            {
                Ok(skill) => ToolResult::ok(format!(
                    "Skill loaded. Follow its instructions:\n\n{content}",
                    content = skill.content
                ))
                .with_display_summary(skill.name),
                Err(error) => super::tool_dispatcher_error::skill_load(error),
            }
        }
        name if name == super::tool_extension_resource::NAME => {
            super::tool_extension_resource::execute(args, session_id).await
        }
        "manage_automation" => super::tool_automation::execute(args, working_dir, session_id).await,
        "create_branch" => {
            let branch_name = args["branch_name"].as_str().unwrap_or("");
            if branch_name.is_empty() {
                return ToolResult::validation(
                    "branch_name_required",
                    "Paramètre branch_name requis",
                );
            }
            match crate::services::git::branch::create_branch(working_dir, branch_name) {
                Ok(()) => ToolResult::ok(format!(
                    "Branche '{}' créée et activée dans {}",
                    branch_name,
                    working_dir.display()
                )),
                Err(error) => super::tool_git_error::create_branch(error),
            }
        }
        "checkout_branch" => {
            let branch_name = args["branch_name"].as_str().unwrap_or("");
            if branch_name.is_empty() {
                return ToolResult::validation(
                    "branch_name_required",
                    "Paramètre branch_name requis",
                );
            }
            match crate::services::git::branch::checkout_branch(working_dir, branch_name) {
                Ok(()) => ToolResult::ok(format!("Basculé sur la branche '{}'", branch_name)),
                Err(error) => super::tool_git_error::checkout_branch(error),
            }
        }
        "delegate_task" => {
            super::tool_dispatcher_delegate::dispatch_delegate(args, session_id, cancel.clone())
                .await
        }
        _ => {
            super::tool_dispatcher_fallback::dispatch(
                tool_name,
                args,
                working_dir,
                session_id,
                cancel,
            )
            .await
        }
    }
}
