pub(crate) const SUBAGENT_READ_ONLY: &str = "subagent-read-only";

pub(crate) async fn ensure_allowed(session_id: &str) -> Result<(), String> {
    let session = super::session_store::get(session_id)
        .await
        .map_err(|_| "session-unavailable".to_string())?;
    if session.parent_session_id.is_some() {
        return Err(SUBAGENT_READ_ONLY.to_string());
    }
    Ok(())
}
