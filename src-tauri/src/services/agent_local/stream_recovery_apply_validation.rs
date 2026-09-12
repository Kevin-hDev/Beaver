use super::conversation_interrupted_tail::CloseInterruptedTailError;
use super::stream_recovery_apply::{OwnerTerminal, StreamRecoveryMode};
use super::stream_recovery_projection::RecoveryProjection;
use super::types_session::AgentSession;

pub(super) fn path_matches(
    session_id: &str,
    path: &std::path::Path,
    projection: &RecoveryProjection,
) -> bool {
    projection.header.session_id == session_id
        && request_from_path(path).as_deref() == Some(&projection.header.request_id)
}

pub(super) fn should_skip_live(
    projection: &RecoveryProjection,
    mode: &StreamRecoveryMode<'_>,
) -> bool {
    if !super::stream_recovery_owners::is_live(
        &projection.header.session_id,
        &projection.header.request_id,
    ) {
        return false;
    }
    !matches!(mode, StreamRecoveryMode::Owner { request_id, .. } if *request_id == projection.header.request_id)
}

pub(super) fn owner_matches(session: &AgentSession, projection: &RecoveryProjection) -> bool {
    match &projection.header.subagent_owner {
        Some(owner) => session.subagent_run_id.as_deref() == Some(&owner.run_id),
        None => true,
    }
}

pub(super) fn superseded(session: &AgentSession, projection: &RecoveryProjection) -> bool {
    let Some(user_index) = session.messages.iter().position(|message| {
        message.id == projection.header.user_message_id
            && message.turn_id == projection.header.turn_id
            && message.role == "user"
    }) else {
        return true;
    };
    projection.header.subagent_owner.is_none()
        && session.messages[user_index + 1..]
            .iter()
            .any(|message| message.role == "user" && message.turn_id != projection.header.turn_id)
}

pub(super) fn apply_projection(
    session: &mut AgentSession,
    projection: &RecoveryProjection,
) -> Result<bool, String> {
    let mut changed = false;
    for message in &projection.messages {
        if let Some(existing) = session.messages.iter().find(|existing| existing.id == message.id) {
            if serde_json::to_value(existing).ok() != serde_json::to_value(message).ok() {
                return Err(error());
            }
            continue;
        }
        if session.messages.len() >= super::session_limits::MAX_MESSAGES_PER_SESSION {
            return Err("session_capacity_reached".into());
        }
        session.messages.push(message.clone());
        changed = true;
    }
    if projection.turn_ready {
        for message in &mut session.messages {
            if message.stream_run_id.as_deref() == Some(&projection.header.request_id)
                && message.stream_part.as_deref() != Some("final")
            {
                message.stream_part = Some("final".into());
                changed = true;
            }
        }
    }
    Ok(changed)
}

pub(super) fn mark_terminal(
    session: &mut AgentSession,
    request_id: &str,
    mode: &StreamRecoveryMode<'_>,
) {
    match mode {
        StreamRecoveryMode::Owner {
            request_id: owner_id,
            terminal,
        } if *owner_id == request_id => match terminal {
            OwnerTerminal::Cancelled => {
                super::stream_diagnostics::apply_cancelled(session, request_id)
            }
            OwnerTerminal::Failed { code } => {
                super::stream_diagnostics::apply_recovered_failure(session, request_id, code)
            }
        },
        _ => super::stream_diagnostics::apply_recovered_failure(
            session,
            request_id,
            "stream_interrupted",
        ),
    }
}

pub(super) fn map_tail_error(error: CloseInterruptedTailError) -> String {
    match error {
        CloseInterruptedTailError::History => self::error(),
        CloseInterruptedTailError::Capacity => "session_capacity_reached".into(),
    }
}

fn request_from_path(path: &std::path::Path) -> Option<String> {
    let name = path.file_name()?.to_str()?;
    let name = name.strip_suffix(".recovering").unwrap_or(name);
    name.strip_suffix(".jsonl").map(str::to_owned)
}

fn error() -> String {
    "stream_recovery_unavailable".into()
}
