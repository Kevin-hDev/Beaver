use crate::services::reasoning_continuity::contract::{
    ContinuationUse, CredentialScope, ReasoningModeId, ReplayTarget, RouteId,
};

pub(crate) async fn resolve(
    session_id: &str,
    provider: &str,
    model: &str,
    _reasoning_mode_hint: Option<&str>,
    _supports_thinking_hint: Option<bool>,
) -> Result<ReplayTarget, String> {
    let session = crate::services::agent_local::session_store::get(session_id)
        .await
        .map_err(|_| generic_error())?;
    if session.provider != provider || session.model != model {
        return Err(generic_error());
    }
    let route_id = RouteId::from_provider_id(provider).ok_or_else(generic_error)?;
    let scope =
        crate::services::api_keys::credential_scope(route_id).map_err(|_| generic_error())?;
    build_with_scope(model, session.reasoning_mode.as_deref(), route_id, scope)
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
mod tests {
    use super::*;

    #[test]
    fn api_and_oauth_routes_keep_distinct_scopes_without_exposing_them() {
        let api = build_with_scope(
            "grok-4.6",
            Some("high"),
            RouteId::Xai,
            CredentialScope::authenticated("api-scope").unwrap(),
        )
        .unwrap();
        let oauth = build_with_scope(
            "grok-4.6",
            Some("high"),
            RouteId::XaiOauth,
            CredentialScope::authenticated("oauth-scope").unwrap(),
        )
        .unwrap();

        assert_eq!(api.route_id, RouteId::Xai);
        assert_eq!(oauth.route_id, RouteId::XaiOauth);
        assert_ne!(api.credential_scope, oauth.credential_scope);
        assert_eq!(api.reasoning_mode, ReasoningModeId::High);
    }

    #[test]
    fn unknown_routes_and_modes_fail_closed() {
        assert!(RouteId::from_provider_id("forged").is_none());
        assert!(reasoning_mode_id(Some("forged")).is_err());
    }

    #[tokio::test]
    async fn forged_frontend_hints_cannot_change_persisted_reasoning_mode() {
        let mut session = crate::services::agent_local::session_store::create_full(
            "Canonical target",
            "qwen3.5:4b",
            "ollama",
            false,
            None,
        )
        .await
        .unwrap();
        session.reasoning_mode = Some("high".to_string());
        crate::services::agent_local::session_store::save(&session)
            .await
            .unwrap();

        let target = resolve(
            &session.id,
            "ollama",
            "qwen3.5:4b",
            Some("off"),
            Some(false),
        )
        .await
        .unwrap();
        assert_eq!(target.reasoning_mode, ReasoningModeId::High);
        assert!(resolve(
            &session.id,
            "xai-oauth",
            "qwen3.5:4b",
            Some("high"),
            Some(true),
        )
        .await
        .is_err());
        crate::services::agent_local::session_store::delete_one(&session.id)
            .await
            .unwrap();
    }
}
