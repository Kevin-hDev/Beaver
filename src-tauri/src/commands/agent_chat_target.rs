use crate::services::reasoning_continuity::contract::{
    ContinuationTarget, ContinuationUse, CredentialScope, NonReplayTarget, ReasoningModeId,
    ReplayTarget, RouteId,
};

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
        Some(crate::services::api_keys::credential_scope(route_id).map_err(|_| generic_error())?)
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
    let continuation = if route_id == RouteId::Groq {
        let target = NonReplayTarget {
            route_id,
            model_id: session.model.clone(),
            reasoning_mode: reasoning.mode,
        };
        target.validate().map_err(|_| generic_error())?;
        ContinuationTarget::Forbidden(target)
    } else {
        let scope = credential_scope.ok_or_else(generic_error)?;
        ContinuationTarget::Replay(build_with_mode(
            &session.model,
            route_id,
            scope,
            reasoning.mode,
        )?)
    };
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

#[cfg(test)]
fn build_with_scope(
    model: &str,
    persisted_mode: Option<&str>,
    route_id: RouteId,
    credential_scope: CredentialScope,
) -> Result<ReplayTarget, String> {
    let mode = reasoning_mode_id(persisted_mode)?;
    build_with_mode(model, route_id, credential_scope, mode)
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

#[cfg(test)]
fn reasoning_mode_id(mode: Option<&str>) -> Result<ReasoningModeId, String> {
    ReasoningModeId::from_name(mode).ok_or_else(generic_error)
}

fn generic_error() -> String {
    "conversation_admission_failed".to_string()
}

#[cfg(test)]
#[path = "agent_chat_target_tests.rs"]
mod tests;
