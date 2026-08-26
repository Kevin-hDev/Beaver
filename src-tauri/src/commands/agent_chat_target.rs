use crate::services::reasoning_continuity::contract::{
    ContinuationTarget, ContinuationUse, CredentialScope, NonReplayTarget, ReasoningModeId,
    ReplayTarget, RouteId,
};

pub(crate) struct ResolvedChatTarget {
    pub continuation: ContinuationTarget,
    pub think: bool,
    pub reasoning_mode: Option<String>,
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
        });
    }
    let scope =
        crate::services::api_keys::credential_scope(route_id).map_err(|_| generic_error())?;
    let replay = build_with_scope(model, session.reasoning_mode.as_deref(), route_id, scope)?;
    Ok(ResolvedChatTarget {
        continuation: ContinuationTarget::Replay(replay),
        think: session.thinking_enabled,
        reasoning_mode: session.reasoning_mode,
    })
}

fn build_with_scope(
    model: &str,
    persisted_mode: Option<&str>,
    route_id: RouteId,
    credential_scope: CredentialScope,
) -> Result<ReplayTarget, String> {
    let target = ReplayTarget {
        route_id,
        model_id: model.to_string(),
        credential_scope,
        reasoning_mode: reasoning_mode_id(persisted_mode)?,
        continuation_use: ContinuationUse::UserContinuation,
    };
    target.validate().map_err(|_| generic_error())?;
    Ok(target)
}

fn reasoning_mode_id(mode: Option<&str>) -> Result<ReasoningModeId, String> {
    match mode.unwrap_or("off") {
        "off" => Ok(ReasoningModeId::Off),
        "auto" => Ok(ReasoningModeId::Auto),
        "low" => Ok(ReasoningModeId::Low),
        "medium" => Ok(ReasoningModeId::Medium),
        "high" => Ok(ReasoningModeId::High),
        "xhigh" => Ok(ReasoningModeId::Xhigh),
        "max" => Ok(ReasoningModeId::Max),
        "ultra" => Ok(ReasoningModeId::Ultra),
        _ => Err(generic_error()),
    }
}

fn generic_error() -> String {
    "conversation_admission_failed".to_string()
}

#[cfg(test)]
#[path = "agent_chat_target_tests.rs"]
mod tests;
