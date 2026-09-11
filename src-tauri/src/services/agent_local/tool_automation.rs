#[path = "tool_automation_view.rs"]
mod view;

use super::tool_automation_validation::{self as validation, Action, TargetMode};
use crate::models::AutomationTarget;
use crate::services::agent_local::tool_result_contract::ToolErrorCategory;
use crate::services::agent_local::types_tools::ToolResult;
use crate::services::automations::{AutomationActor, AutomationError, CreateAutomation};
use serde_json::{json, Value};
use std::path::Path;

#[cfg(test)]
pub(crate) static AUTOMATION_TOOL_TEST_LOCK: tokio::sync::Mutex<()> =
    tokio::sync::Mutex::const_new(());

pub async fn execute(
    args: &Value,
    _working_dir: &Path,
    session_id: &str,
    request_id: Option<&str>,
) -> ToolResult {
    let action_name = action_name(args);
    let request = match validation::parse(args) {
        Ok(request) => request,
        Err(code) => return failure(action_name, code, ToolErrorCategory::Validation),
    };
    let session = match super::session_store::get(session_id).await {
        Ok(session) => session,
        Err(_) => {
            return failure(
                action_name,
                "store_unavailable",
                ToolErrorCategory::Unavailable,
            )
        }
    };
    let actor = match crate::services::automations::actor_context::actor_for(
        &session.id,
        session.is_gateway,
        session.gateway_channel_key.as_deref(),
        request_id,
    ) {
        Ok(actor) => actor,
        Err(error) => return automation_failure(action_name, error),
    };
    dispatch(request, session, actor).await
}

fn action_name(args: &Value) -> &'static str {
    match args["action"].as_str() {
        Some("list") => "list",
        Some("get") => "get",
        Some("create") => "create",
        Some("update") => "update",
        Some("history") => "history",
        Some("delete") => "delete",
        _ => "unknown",
    }
}

async fn dispatch(
    request: Action,
    session: super::types_session::AgentSession,
    actor: AutomationActor,
) -> ToolResult {
    match request {
        Action::List => match crate::services::automations::list(&actor).await {
            Ok(items) => success("list", view::summaries(items)),
            Err(error) => automation_failure("list", error),
        },
        Action::Get(id) => match crate::services::automations::get(&actor, id).await {
            Ok(item) => success("get", view::detail(item)),
            Err(error) => automation_failure("get", error),
        },
        Action::Create(request) => {
            let target = match request.target_mode {
                TargetMode::NewSession => AutomationTarget::NewSession {
                    project_id: session.project_id.clone(),
                },
                TargetMode::ResumeSession => AutomationTarget::ResumeSession {
                    session_id: session.id.clone(),
                },
            };
            let input = CreateAutomation {
                name: request.name,
                description: request.description,
                prompt: request.prompt,
                target,
                provider: session.provider,
                model: request.model.unwrap_or(session.model),
                schedule: request.schedule,
                status: request.status,
            };
            match crate::services::automations::create(&actor, input).await {
                Ok(item) => {
                    crate::services::scheduler::notify_config_changed();
                    success("create", view::detail(item))
                }
                Err(error) => automation_failure("create", error),
            }
        }
        Action::Update(id, patch) => {
            match crate::services::automations::update(&actor, id, patch).await {
                Ok(item) => {
                    crate::services::scheduler::notify_config_changed();
                    success("update", view::detail(item))
                }
                Err(error) => automation_failure("update", error),
            }
        }
        Action::History(query) => {
            match crate::services::automations::history(&actor, query).await {
                Ok(page) => success("history", view::history(page)),
                Err(error) => automation_failure("history", error),
            }
        }
        Action::Delete(id) => match crate::services::automations::delete(&actor, id).await {
            Ok(()) => {
                crate::services::scheduler::notify_config_changed();
                success("delete", json!({"automation_id": id}))
            }
            Err(error) => automation_failure("delete", error),
        },
    }
}

fn success(action: &str, data: Value) -> ToolResult {
    ToolResult::ok(json!({"ok":true, "action":action, "data":data}).to_string())
}

fn automation_failure(action: &str, error: AutomationError) -> ToolResult {
    failure(action, error.code(), error_category(error))
}

fn failure(action: &str, code: &'static str, category: ToolErrorCategory) -> ToolResult {
    ToolResult::error(
        json!({"ok":false, "action":action, "error_code":code}).to_string(),
        code,
        category,
        false,
    )
}

fn error_category(error: AutomationError) -> ToolErrorCategory {
    match error {
        AutomationError::NotFound => ToolErrorCategory::NotFound,
        AutomationError::InvalidInput
        | AutomationError::ImmutableField
        | AutomationError::InvalidSchedule => ToolErrorCategory::Validation,
        AutomationError::CapacityReached => ToolErrorCategory::Conflict,
        _ => ToolErrorCategory::Unavailable,
    }
}
