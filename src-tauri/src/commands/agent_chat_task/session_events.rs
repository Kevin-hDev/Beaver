use crate::services::agent_local::agent_loop_finish::CompletedStreamTurn;

pub fn emit_started(session_id: &str, request_id: &str, is_chat: bool) -> bool {
    !is_chat && crate::services::extensions::turn_started(session_id, request_id)
}

pub fn emit_terminal(
    session_id: &str,
    request_id: &str,
    result: &Result<CompletedStreamTurn, String>,
) {
    let (event, status) = match result {
        Ok(_) => ("session.turn.completed", "completed"),
        Err(error) if error == "Annulé" => ("session.turn.cancelled", "cancelled"),
        Err(_) => ("session.turn.failed", "failed"),
    };
    let _ = crate::services::extensions::turn_terminal(session_id, request_id, event, status);
}

#[cfg(test)]
mod tests {
    #[test]
    fn chat_has_no_extension_observer_events() {
        assert!(!super::emit_started("session", "request", true));
    }
}
