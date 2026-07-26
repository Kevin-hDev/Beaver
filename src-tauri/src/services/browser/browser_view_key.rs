use super::tab_id::validate_tab_id;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct BrowserViewKey {
    pub(super) session_id: String,
    pub(super) tab_id: String,
}

impl BrowserViewKey {
    pub(super) fn new(session_id: String, tab_id: String) -> Result<Self, ()> {
        crate::services::agent_local::session_store::validate_session_id(&session_id)
            .map_err(|_| ())?;
        validate_tab_id(&tab_id)?;
        Ok(Self { session_id, tab_id })
    }
}
