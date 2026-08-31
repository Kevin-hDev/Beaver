use super::checkpoint_transaction::CompressionError;
use super::profile_resolve::ResolvedCompressionProfile;
use super::profile_types::CompressionTrigger;
use crate::services::agent_local::types_message::AgentMessageKind;
use crate::services::agent_local::types_session::{
    AgentSession, AutomaticCompressionAttempt, AutomaticCompressionGuard,
};

pub struct PreparedGuard {
    pub session: AgentSession,
    pub attempt: Option<AutomaticCompressionAttempt>,
}

pub async fn prepare(
    expected: &AgentSession,
    profile: &ResolvedCompressionProfile,
    context_window: u64,
    trigger: CompressionTrigger,
) -> Result<Option<PreparedGuard>, CompressionError> {
    let lock = crate::services::agent_local::session_store::lock_session(&expected.id).await;
    let _guard = lock.lock().await;
    let mut current = crate::services::agent_local::session_store::get(&expected.id)
        .await
        .map_err(|_| CompressionError::SaveFailed)?;
    if !super::checkpoint_candidate::same_messages(&current.messages, &expected.messages)
        || current.model != expected.model
        || current.provider != expected.provider
    {
        return Err(CompressionError::SessionChanged);
    }
    let resolved = super::profile_resolve::resolve_for_session(&current)
        .map_err(|_| CompressionError::Unavailable)?;
    if resolved.profile.id != profile.profile.id
        || resolved.profile_revision != profile.profile_revision
        || resolved.global_selection_revision != profile.global_selection_revision
    {
        return Err(CompressionError::SessionChanged);
    }
    if trigger == CompressionTrigger::Explicit {
        current.automatic_compression_guard = AutomaticCompressionGuard::default();
        crate::services::agent_local::session_store::save(&current)
            .await
            .map_err(|_| CompressionError::SaveFailed)?;
        return Ok(Some(PreparedGuard {
            session: current,
            attempt: None,
        }));
    }
    let attempt = attempt_for(&current, profile, context_window)?;
    match start(&mut current.automatic_compression_guard, &attempt) {
        StartDecision::AlreadyAttempted => return Ok(None),
        StartDecision::Suspended => return Err(CompressionError::AutomaticSuspended),
        StartDecision::Proceed => {}
    }
    crate::services::agent_local::session_store::save(&current)
        .await
        .map_err(|_| CompressionError::SaveFailed)?;
    Ok(Some(PreparedGuard {
        session: current,
        attempt: Some(attempt),
    }))
}

pub async fn record_failure(session_id: &str, attempt: &AutomaticCompressionAttempt) {
    let lock = crate::services::agent_local::session_store::lock_session(session_id).await;
    let _guard = lock.lock().await;
    let Ok(mut session) = crate::services::agent_local::session_store::get(session_id).await else {
        log::warn!("automatic_compression_guard_load_failed");
        return;
    };
    if session.automatic_compression_guard.last_attempt.as_ref() != Some(attempt) {
        return;
    }
    fail(&mut session.automatic_compression_guard);
    if crate::services::agent_local::session_store::save(&session)
        .await
        .is_err()
    {
        log::warn!("automatic_compression_guard_save_failed");
    }
}

pub fn success_guard(
    attempt: Option<AutomaticCompressionAttempt>,
    after_tokens: u32,
    context_window: u64,
    threshold_percent: u8,
) -> AutomaticCompressionGuard {
    let Some(attempt) = attempt else {
        return AutomaticCompressionGuard::default();
    };
    let still_above = super::token_estimate::should_compress(
        after_tokens as usize,
        context_window,
        threshold_percent,
    );
    AutomaticCompressionGuard {
        last_attempt: still_above.then_some(attempt),
        consecutive_failures: 0,
        suspended: false,
    }
}

pub fn allows_realtime(
    session: &AgentSession,
    profile: &ResolvedCompressionProfile,
    context_window: u64,
) -> bool {
    let Ok(attempt) = attempt_for(session, profile, context_window) else {
        return false;
    };
    let guard = &session.automatic_compression_guard;
    if guard
        .last_attempt
        .as_ref()
        .is_some_and(|previous| !same_environment(previous, &attempt))
    {
        return true;
    }
    !guard.suspended && guard.last_attempt.as_ref() != Some(&attempt)
}

pub fn reset(session: &mut AgentSession) {
    session.automatic_compression_guard = AutomaticCompressionGuard::default();
}

fn attempt_for(
    session: &AgentSession,
    profile: &ResolvedCompressionProfile,
    context_window: u64,
) -> Result<AutomaticCompressionAttempt, CompressionError> {
    let last = session
        .messages
        .last()
        .ok_or(CompressionError::SnapshotInvalid)?;
    let last_checkpoint_message_id = session
        .messages
        .iter()
        .rev()
        .find(|message| message.message_kind == Some(AgentMessageKind::CompressionCheckpoint))
        .map(|message| message.id.clone());
    Ok(AutomaticCompressionAttempt {
        top_level_turn_id: last.turn_id.clone(),
        last_message_id: last.id.clone(),
        message_count: u16::try_from(session.messages.len())
            .map_err(|_| CompressionError::SnapshotInvalid)?,
        last_checkpoint_message_id,
        provider_id: session.provider.clone(),
        model_id: session.model.clone(),
        context_window,
        profile_id: profile.profile.id.clone(),
        profile_revision: profile.profile_revision,
        global_selection_revision: profile.global_selection_revision,
    })
}

enum StartDecision {
    Proceed,
    AlreadyAttempted,
    Suspended,
}

fn start(
    guard: &mut AutomaticCompressionGuard,
    attempt: &AutomaticCompressionAttempt,
) -> StartDecision {
    if guard
        .last_attempt
        .as_ref()
        .is_some_and(|previous| !same_environment(previous, attempt))
    {
        *guard = AutomaticCompressionGuard::default();
    }
    if guard.suspended {
        return StartDecision::Suspended;
    }
    if guard.last_attempt.as_ref() == Some(attempt) {
        return StartDecision::AlreadyAttempted;
    }
    guard.last_attempt = Some(attempt.clone());
    StartDecision::Proceed
}

fn fail(guard: &mut AutomaticCompressionGuard) {
    guard.consecutive_failures = guard.consecutive_failures.saturating_add(1).min(3);
    guard.suspended = guard.consecutive_failures >= 3;
}

fn same_environment(
    left: &AutomaticCompressionAttempt,
    right: &AutomaticCompressionAttempt,
) -> bool {
    left.provider_id == right.provider_id
        && left.model_id == right.model_id
        && left.context_window == right.context_window
        && left.profile_id == right.profile_id
        && left.profile_revision == right.profile_revision
        && left.global_selection_revision == right.global_selection_revision
}

#[cfg(test)]
#[path = "automatic_guard_tests.rs"]
mod tests;
