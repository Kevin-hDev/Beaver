use serde::Serialize;

use crate::models::agent_session_contract::{
    AgentSessionView, AgentStreamFailureView, ContinuityCapability, ReasoningReplayStatus,
    SubagentLastActivityView,
};
use crate::services::reasoning_continuity::contract::{ContinuationUse, ReasoningModeId, RouteId};
use crate::services::reasoning_continuity::envelope::{CompletionState, ReasoningEnvelope};
use crate::services::reasoning_continuity::registry::{ActivationState, ReplayRequirement};

use super::types_session::{AgentSession, CloneMode};

// Remote attend une preuve dédiée `previous_response_id`; les fixtures locales
// d'items Responses ne prouvent pas cette continuité côté fournisseur.
pub const REMOTE_PREVIOUS_RESPONSE_CONTINUATION_AVAILABLE: bool = false;

pub fn from_session(session: &AgentSession) -> Result<AgentSessionView, String> {
    Ok(AgentSessionView {
        id: session.id.clone(),
        name: session.name.clone(),
        created_at: session.created_at,
        updated_at: session.updated_at,
        archived_at: session.archived_at,
        pinned_at: session.pinned_at,
        model: session.model.clone(),
        provider: session.provider.clone(),
        thinking_enabled: session.thinking_enabled,
        fast_mode_enabled: session.fast_mode_enabled,
        reasoning_mode: session.reasoning_mode.clone(),
        preserve_reasoning: effective_preserve_reasoning(session),
        continuity_capability: continuity_capability(session),
        accumulated_tokens: session.accumulated_tokens,
        context_tokens: session.context_tokens,
        messages: session
            .messages
            .iter()
            .map(super::session_view_message::from_message)
            .collect::<Result<Vec<_>, _>>()?,
        todos: json_values(&session.todos)?,
        todo_runs: json_values(&session.todo_runs)?,
        active_todo_run_id: session.active_todo_run_id.clone(),
        stream_failures: session
            .stream_failures
            .iter()
            .map(|failure| AgentStreamFailureView {
                code: failure.code.clone(),
                occurred_at: failure.occurred_at,
                is_connection: failure.is_connection,
                active_todo_run_id: failure.active_todo_run_id.clone(),
                active_todo_title: failure.active_todo_title.clone(),
            })
            .collect(),
        diagnostic_runs: json_values(&session.diagnostic_runs)?,
        plan_mode_enabled: session.plan_mode_enabled,
        plan_runs: json_values(&session.plan_runs)?,
        active_plan_id: session.active_plan_id.clone(),
        plan_workflow_status: json_value(&session.plan_workflow_status)?,
        is_heartbeat: session.is_heartbeat,
        is_gateway: session.is_gateway,
        gateway_channel_key: session.gateway_channel_key.clone(),
        project_id: session.project_id.clone(),
        working_dir: session.working_dir.clone(),
        working_dir_managed: session.working_dir_managed,
        parent_session_id: session.parent_session_id.clone(),
        subagent_type: session.subagent_type.clone(),
        subagent_worktree: session.subagent_worktree.clone(),
        subagent_status: session.subagent_status.clone(),
        subagent_run_id: session.subagent_run_id.clone(),
        subagent_description: session.subagent_description.clone(),
        subagent_color_key: session.subagent_color_key.clone(),
        subagent_summary: session.subagent_summary.clone(),
        subagent_last_activity: session.subagent_last_activity.as_ref().map(|activity| {
            SubagentLastActivityView {
                kind: activity.kind.clone(),
                label: activity.label.clone(),
                detail: activity.detail.clone(),
                updated_at: activity.updated_at,
            }
        }),
        clone_parent_session_id: session.clone_parent_session_id.clone(),
        clone_parent_message_id: session.clone_parent_message_id.clone(),
        clone_mode: session.clone_mode.as_ref().map(|mode| match mode {
            CloneMode::Cut => "cut".to_string(),
            CloneMode::Summary => "summary".to_string(),
        }),
        clone_root_session_id: session.clone_root_session_id.clone(),
        git_branch: session.git_branch.clone(),
    })
}

fn effective_preserve_reasoning(
    session: &AgentSession,
) -> super::types_session::PreserveReasoningSetting {
    let Some(capability) = continuity_capability(session) else {
        return session.preserve_reasoning;
    };
    if capability.requirement == "required"
        && session.preserve_reasoning == super::types_session::PreserveReasoningSetting::Off
    {
        // Une route qui exige la continuité ne peut jamais exposer une option
        // sélectionnée mais interdite : Local est la valeur déterministe.
        super::types_session::PreserveReasoningSetting::Local
    } else if session.preserve_reasoning == super::types_session::PreserveReasoningSetting::Remote
        && !capability.remote_available
    {
        // Une ancienne préférence Remote ne doit jamais laisser l'interface
        // afficher un choix devenu indisponible.
        super::types_session::PreserveReasoningSetting::Local
    } else {
        session.preserve_reasoning
    }
}

/// Le registre est l'autorité des capacités : le client ne reconstruit jamais
/// ce choix depuis le fournisseur affiché ou `supports_thinking`.
pub fn continuity_capability(session: &AgentSession) -> Option<ContinuityCapability> {
    let route = RouteId::from_provider_id(&session.provider)?;
    let mode = ReasoningModeId::from_name(session.reasoning_mode.as_deref())?;
    let policies = crate::services::reasoning_continuity::registry::active_routes()
        .iter()
        .find(|entry| entry.route_id == route)?
        .models
        .iter()
        .filter(|policy| policy.model_id == session.model && policy.reasoning_mode == mode)
        .filter(|policy| {
            matches!(
                policy.continuation_use,
                ContinuationUse::UserContinuation | ContinuationUse::ToolContinuation
            )
        })
        .collect::<Vec<_>>();
    if policies.is_empty()
        || policies
            .iter()
            .any(|policy| policy.activation != ActivationState::LiveValidated)
    {
        return None;
    }

    let requirement = if policies
        .iter()
        .any(|policy| policy.requirement == ReplayRequirement::Required)
    {
        ReplayRequirement::Required
    } else if policies
        .iter()
        .any(|policy| policy.requirement == ReplayRequirement::Optional)
    {
        ReplayRequirement::Optional
    } else {
        ReplayRequirement::Forbidden
    };
    if requirement == ReplayRequirement::Forbidden {
        return None;
    }
    let remote_available = REMOTE_PREVIOUS_RESPONSE_CONTINUATION_AVAILABLE
        && policies
            .iter()
            .all(|policy| policy.fixture_id.is_some() && policy.fixture_date.is_some());
    let (requirement, state, explanation_key) = match requirement {
        ReplayRequirement::Required => ("required", "locked", "agentLocal.continuityRequired"),
        ReplayRequirement::Optional => ("optional", "available", "agentLocal.continuityOptional"),
        ReplayRequirement::Forbidden => return None,
    };
    Some(ContinuityCapability {
        requirement: requirement.to_string(),
        local_available: true,
        remote_available,
        state: state.to_string(),
        explanation_key: explanation_key.to_string(),
    })
}

pub(crate) fn messages(
    source: &[super::types_message::AgentMessage],
) -> Result<Vec<crate::models::agent_session_contract::AgentMessageView>, String> {
    source
        .iter()
        .map(super::session_view_message::from_message)
        .collect()
}

pub(super) fn replay_status(envelope: Option<&ReasoningEnvelope>) -> ReasoningReplayStatus {
    let Some(envelope) = envelope else {
        return ReasoningReplayStatus::Unavailable;
    };
    match envelope.completion {
        CompletionState::Partial => ReasoningReplayStatus::Partial,
        CompletionState::Compacted if envelope.validate().is_ok() => {
            ReasoningReplayStatus::Compacted
        }
        CompletionState::Compacted => ReasoningReplayStatus::Unavailable,
        CompletionState::Complete if envelope.validate().is_ok() => {
            ReasoningReplayStatus::Preserved
        }
        CompletionState::Complete => ReasoningReplayStatus::Unavailable,
    }
}

fn json_values<T: Serialize>(items: &[T]) -> Result<Vec<serde_json::Value>, String> {
    items.iter().map(json_value).collect()
}

fn json_value<T: Serialize>(value: &T) -> Result<serde_json::Value, String> {
    serde_json::to_value(value).map_err(|_| "Session indisponible".to_string())
}
