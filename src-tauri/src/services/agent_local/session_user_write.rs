pub(crate) const SUBAGENT_READ_ONLY: &str = "subagent-read-only";

/// Refuse toute écriture IPC dans une session enfant : son unique auteur est le
/// runtime du sous-agent. Le document fait foi, l'index n'étant qu'un cache
/// reconstructible ; un clone reste éditable car `clone_parent_session_id` n'est pas ce lien.
pub(crate) async fn ensure_allowed(session_id: &str) -> Result<(), String> {
    let session = super::session_store::get(session_id).await?;
    if session.parent_session_id.is_some() {
        return Err(SUBAGENT_READ_ONLY.to_string());
    }
    Ok(())
}
