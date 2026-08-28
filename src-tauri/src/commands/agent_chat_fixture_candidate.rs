//! Autorisation temporaire et locale d'un couple de fixture non encore activé.

use crate::services::reasoning_continuity::contract::{
    ContinuationTarget, ContinuationUse, ReplayTarget, RouteId,
};
use crate::services::reasoning_continuity::registry::ReplayRequirement;

pub(crate) async fn resolve(
    session_id: &str,
    provider: &str,
    model: &str,
    reasoning_mode_hint: Option<&str>,
    supports_thinking_hint: Option<bool>,
    fixture_run: &crate::services::reasoning_fixture_run::FixtureRunContext,
) -> Result<super::agent_chat_target::ResolvedChatTarget, String> {
    // La possession du contexte, créé seulement par l'IPC de fixture debug,
    // est la capacité qui empêche ce chemin d'exister pour le chat normal.
    let _ = fixture_run;
    let mut resolved = super::agent_chat_target::resolve(
        session_id,
        provider,
        model,
        reasoning_mode_hint,
        supports_thinking_hint,
    )
    .await?;
    if resolved.continuation.replay().is_some() {
        return Ok(resolved);
    }
    let session = crate::services::agent_local::session_store::get(session_id)
        .await
        .map_err(|_| error())?;
    let route_id = RouteId::from_provider_id(provider).ok_or_else(error)?;
    if session.provider != provider || session.model != model {
        return Ok(resolved);
    }
    let target = ReplayTarget {
        route_id,
        model_id: model.to_owned(),
        credential_scope: crate::services::api_keys::credential_scope(route_id)
            .map_err(|_| error())?,
        reasoning_mode: resolved.reasoning.mode,
        continuation_use: ContinuationUse::UserContinuation,
    };
    target.validate().map_err(|_| error())?;
    let Some(policy) = crate::services::reasoning_continuity::registry::replay_policy(&target)
    else {
        return Ok(resolved);
    };
    if policy.requirement() != ReplayRequirement::Forbidden {
        resolved.continuation = ContinuationTarget::FixtureCandidate(target);
    }
    Ok(resolved)
}

fn error() -> String {
    "conversation_admission_failed".to_string()
}
