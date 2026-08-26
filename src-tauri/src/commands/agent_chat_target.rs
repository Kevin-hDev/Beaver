use crate::services::reasoning_continuity::contract::{
    ContinuationTarget, ContinuationUse, CredentialScope, NonReplayTarget, ReasoningModeId,
    ReplayTarget, RouteId,
};
use crate::services::reasoning_continuity::registry::{ActivationState, ReplayRequirement};

pub(crate) struct ResolvedChatTarget {
    pub continuation: ContinuationTarget,
    pub reasoning: crate::services::reasoning_profile::EffectiveReasoningProfile,
    pub session_reasoning:
        crate::services::agent_local::conversation_reasoning_state::SessionReasoningUpdate,
}

pub(crate) async fn resolve(
    session_id: &str,
    provider: &str,
    model: &str,
    _reasoning_mode_hint: Option<&str>,
    _supports_thinking_hint: Option<bool>,
) -> Result<ResolvedChatTarget, String> {
    let session = crate::services::agent_local::session_store::get(session_id)
        .await
        .map_err(|_| generic_error())?;
    if session.provider != provider || session.model != model {
        return Err(generic_error());
    }
    let route_id = RouteId::from_provider_id(provider).ok_or_else(generic_error)?;
    let ollama_capabilities = if route_id == RouteId::Ollama {
        Some(
            crate::services::agent_local::ollama_client::OllamaClient::from_global()
                .map_err(|_| generic_error())?
                .show_model(model)
                .await
                .map_err(|_| generic_error())?
                .capabilities,
        )
    } else {
        None
    };
    let supports_api_thinking = if route_id == RouteId::Ollama {
        None
    } else {
        Some(
            super::agent_chat_task::api_capabilities::resolve(provider, model, &Default::default())
                .await
                .thinking,
        )
    };
    let scope = if route_id == RouteId::Groq {
        None
    } else {
        crate::services::api_keys::credential_scope(route_id).ok()
    };
    resolve_session(
        session,
        route_id,
        ollama_capabilities.as_deref(),
        supports_api_thinking,
        scope,
    )
}

fn resolve_session(
    session: crate::services::agent_local::types_session::AgentSession,
    route_id: RouteId,
    ollama_capabilities: Option<&[String]>,
    supports_api_thinking: Option<bool>,
    credential_scope: Option<CredentialScope>,
) -> Result<ResolvedChatTarget, String> {
    let reasoning = if route_id == RouteId::Ollama {
        crate::services::reasoning_profile::EffectiveReasoningProfile::ollama(
            &session.model,
            session.reasoning_mode.as_deref(),
            session.thinking_enabled,
            ollama_capabilities,
        )
    } else {
        let supports_thinking = supports_api_thinking.ok_or_else(generic_error)?;
        crate::services::reasoning_profile::EffectiveReasoningProfile::api(
            &session.provider,
            &session.model,
            session.reasoning_mode.as_deref(),
            session.thinking_enabled,
            supports_thinking,
        )
    }
    .map_err(|_| generic_error())?;
    let continuation =
        continuation_for_session(&session, route_id, credential_scope, reasoning.mode)?;
    let session_reasoning =
        crate::services::agent_local::conversation_reasoning_state::SessionReasoningUpdate::new(
            &session, &reasoning,
        );
    Ok(ResolvedChatTarget {
        continuation,
        reasoning,
        session_reasoning,
    })
}

fn continuation_for_session(
    session: &crate::services::agent_local::types_session::AgentSession,
    route_id: RouteId,
    credential_scope: Option<CredentialScope>,
    reasoning_mode: ReasoningModeId,
) -> Result<ContinuationTarget, String> {
    crate::services::reasoning_continuity::limits::validate_model_id(&session.model)
        .map_err(|_| generic_error())?;
    let blocked = || {
        Ok(ContinuationTarget::Forbidden(NonReplayTarget {
            route_id,
            model_id: session.model.clone(),
            reasoning_mode,
        }))
    };
    if route_id == RouteId::Groq {
        return blocked();
    }
    let Some(credential_scope) = credential_scope else {
        return blocked();
    };
    let target = match build_with_mode(&session.model, route_id, credential_scope, reasoning_mode) {
        Ok(target) => target,
        Err(_) => return blocked(),
    };
    let mut tool_target = target.clone();
    tool_target.continuation_use = ContinuationUse::ToolContinuation;
    let policies = [
        crate::services::reasoning_continuity::registry::replay_policy(&target),
        crate::services::reasoning_continuity::registry::replay_policy(&tool_target),
    ];
    let required = policies
        .iter()
        .flatten()
        .any(|policy| policy.requirement() == ReplayRequirement::Required);
    let has_live_replay_policy = policies.iter().flatten().any(|policy| {
        policy.activation() == ActivationState::LiveValidated
            && policy.requirement() != ReplayRequirement::Forbidden
    });
    // La capture garde le scope initial jusqu'au tour outil : une politique
    // user forbidden ne doit jamais effacer une politique tool live required.
    let enabled = has_live_replay_policy
        && (required
            || (!matches!(
                session.preserve_reasoning,
                crate::services::agent_local::types_session::PreserveReasoningSetting::Off
            ) && crate::services::reasoning_continuity::registry::replay_policy(&target)
                .is_some()));
    enabled
        .then_some(ContinuationTarget::Replay(target))
        .map_or_else(blocked, Ok)
}

#[cfg(test)]
async fn resolve_with_ollama_capabilities(
    session_id: &str,
    provider: &str,
    model: &str,
    _reasoning_mode_hint: Option<&str>,
    _supports_thinking_hint: Option<bool>,
    capabilities: &[String],
) -> Result<ResolvedChatTarget, String> {
    let session = crate::services::agent_local::session_store::get(session_id)
        .await
        .map_err(|_| generic_error())?;
    if session.provider != provider || session.model != model {
        return Err(generic_error());
    }
    resolve_session(
        session,
        RouteId::Ollama,
        Some(capabilities),
        None,
        Some(CredentialScope::local_uncredentialed()),
    )
}

#[cfg(test)]
async fn resolve_with_api_capability(
    session_id: &str,
    provider: &str,
    model: &str,
    supports_thinking: bool,
) -> Result<ResolvedChatTarget, String> {
    let session = crate::services::agent_local::session_store::get(session_id)
        .await
        .map_err(|_| generic_error())?;
    if session.provider != provider || session.model != model {
        return Err(generic_error());
    }
    let route_id = RouteId::from_provider_id(provider).ok_or_else(generic_error)?;
    let scope = (route_id != RouteId::Groq)
        .then(|| CredentialScope::authenticated("test-scope").map_err(|_| generic_error()))
        .transpose()?;
    resolve_session(session, route_id, None, Some(supports_thinking), scope)
}

fn build_with_mode(
    model: &str,
    route_id: RouteId,
    credential_scope: CredentialScope,
    reasoning_mode: ReasoningModeId,
) -> Result<ReplayTarget, String> {
    let target = ReplayTarget {
        route_id,
        model_id: model.to_string(),
        credential_scope,
        reasoning_mode,
        continuation_use: ContinuationUse::UserContinuation,
    };
    target.validate().map_err(|_| generic_error())?;
    Ok(target)
}

fn generic_error() -> String {
    "conversation_admission_failed".to_string()
}

#[cfg(test)]
#[path = "agent_chat_target_tests.rs"]
mod tests;
