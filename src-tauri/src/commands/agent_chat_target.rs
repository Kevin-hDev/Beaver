use crate::services::reasoning_continuity::contract::{
    ContinuationTarget, ContinuationUse, CredentialScope, NonReplayTarget, ReasoningModeId,
    ReplayTarget, RouteId,
};

pub(crate) struct ResolvedChatTarget {
    pub continuation: ContinuationTarget,
    pub think: bool,
    pub reasoning_mode: Option<String>,
    pub ollama_reasoning: Option<crate::services::reasoning_ollama::EffectiveOllamaReasoning>,
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
    let capabilities =
        if provider == "ollama" && session.provider == provider && session.model == model {
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
    resolve_session(session, provider, model, capabilities.as_deref())
}

fn resolve_session(
    session: crate::services::agent_local::types_session::AgentSession,
    provider: &str,
    model: &str,
    ollama_capabilities: Option<&[String]>,
) -> Result<ResolvedChatTarget, String> {
    if session.provider != provider || session.model != model {
        return Err(generic_error());
    }
    let route_id = RouteId::from_provider_id(provider).ok_or_else(generic_error)?;
    if route_id == RouteId::Ollama {
        let effective = crate::services::reasoning_ollama::resolve(
            model,
            session.reasoning_mode.as_deref(),
            session.thinking_enabled,
            ollama_capabilities,
        )
        .map_err(|_| generic_error())?;
        let scope =
            crate::services::api_keys::credential_scope(route_id).map_err(|_| generic_error())?;
        let replay = build_with_mode(model, route_id, scope, effective.mode)?;
        return Ok(ResolvedChatTarget {
            continuation: ContinuationTarget::Replay(replay),
            think: effective.payload.enabled(),
            reasoning_mode: Some(effective.mode_name.clone()),
            ollama_reasoning: Some(effective),
        });
    }
    if route_id == RouteId::Groq {
        let target = NonReplayTarget {
            route_id,
            model_id: model.to_string(),
            reasoning_mode: reasoning_mode_id(session.reasoning_mode.as_deref())?,
        };
        target.validate().map_err(|_| generic_error())?;
        return Ok(ResolvedChatTarget {
            continuation: ContinuationTarget::Forbidden(target),
            think: session.thinking_enabled,
            reasoning_mode: session.reasoning_mode,
            ollama_reasoning: None,
        });
    }
    let scope =
        crate::services::api_keys::credential_scope(route_id).map_err(|_| generic_error())?;
    let replay = build_with_scope(model, session.reasoning_mode.as_deref(), route_id, scope)?;
    Ok(ResolvedChatTarget {
        continuation: ContinuationTarget::Replay(replay),
        think: session.thinking_enabled,
        reasoning_mode: session.reasoning_mode,
        ollama_reasoning: None,
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
    resolve_session(session, provider, model, Some(capabilities))
}

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

fn reasoning_mode_id(mode: Option<&str>) -> Result<ReasoningModeId, String> {
    ReasoningModeId::from_name(mode).ok_or_else(generic_error)
}

fn generic_error() -> String {
    "conversation_admission_failed".to_string()
}

#[cfg(test)]
#[path = "agent_chat_target_tests.rs"]
mod tests;
