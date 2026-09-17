use crate::services::agent_local::session_store;

pub async fn mark_status(session_id: &str, status: &str) -> Result<(), String> {
    if !super::subagent_status::is_valid(status) {
        return Err("Statut sous-agent invalide".to_string());
    }
    session_store::validate_session_id(session_id)?;
    let lock = session_store::lock_session(session_id).await;
    let _guard = lock.lock().await;
    let mut session = session_store::get(session_id).await?;
    session.subagent_status = Some(status.to_string());
    session_store::save(&session).await
}

#[cfg(test)]
mod tests {
    use super::mark_status;

    #[tokio::test]
    async fn test_reject_invalid_status() {
        // La validation du statut est avant l'accès filesystem.
        let result = mark_status("any-id", "invalid_status").await;
        assert!(result.is_err(), "Un statut invalide doit être rejeté");
        assert_eq!(result.unwrap_err(), "Statut sous-agent invalide");
    }

    #[test]
    fn test_interrupted_status_is_accepted_by_match() {
        let accepted =
            super::super::subagent_status::is_valid(super::super::subagent_status::INTERRUPTED);
        assert!(accepted, "Le statut 'interrupted' doit être accepté");
    }
}
