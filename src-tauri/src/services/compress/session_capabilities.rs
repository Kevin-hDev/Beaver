#![allow(
    dead_code,
    reason = "the compression orchestrator consumes this staged contract in Task 10"
)]

use std::collections::BTreeSet;

use serde::Serialize;

const MAX_RUNTIME_TOOL_NAMES: usize = 256;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SessionCompressionCapabilities {
    pub chatbot: bool,
    pub tool_names: BTreeSet<String>,
    pub project_context: bool,
    pub subagents: bool,
    pub git: bool,
    pub plan_and_tasks: bool,
}

impl SessionCompressionCapabilities {
    pub fn from_runtime(
        chatbot: bool,
        tool_names: &[String],
        project_context: bool,
        git_repository: bool,
        plan_mode_active: bool,
    ) -> Result<Self, String> {
        if tool_names.len() > MAX_RUNTIME_TOOL_NAMES
            || tool_names.iter().any(|name| {
                name.is_empty() || name.len() > 128 || name.chars().any(char::is_control)
            })
        {
            return Err("compression_capabilities_invalid".to_string());
        }
        let tool_names = tool_names.iter().cloned().collect::<BTreeSet<_>>();
        let has = |name: &str| tool_names.contains(name);
        let local_files = [
            "read_file",
            "write_file",
            "edit_file",
            "list_dir",
            "grep",
            "glob",
        ]
        .iter()
        .any(|name| has(name));
        let subagents = crate::services::agent_local::tool_catalog::SUBAGENT_TOOLS
            .iter()
            .any(|name| has(name));
        let plan_and_tasks = [
            "plan_mode",
            "todo_write",
            "todo_history",
            "todo_pause",
            "todo_resume",
            "todo_delete",
        ]
        .iter()
        .any(|name| has(name));
        Ok(Self {
            chatbot,
            project_context: !chatbot && project_context && local_files,
            subagents: !chatbot && subagents,
            git: !chatbot && git_repository,
            plan_and_tasks: !chatbot && plan_mode_active && plan_and_tasks,
            tool_names,
        })
    }
}
