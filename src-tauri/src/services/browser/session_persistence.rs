use super::{
    session_model::SessionModel, session_store, tab_id::new_secure_tab_id, BrowserCommandError,
};

pub(super) fn load_or_create(session_id: &str) -> Result<SessionModel, BrowserCommandError> {
    let key = session_store::session_key().map_err(|_| BrowserCommandError::Unavailable)?;
    let directory = session_store::sessions_dir();
    if let Some(model) = session_store::load_at(&directory, session_id, &key)
        .map_err(|_| BrowserCommandError::Internal)?
    {
        return Ok(model);
    }
    let model =
        SessionModel::new(new_secure_tab_id()).map_err(|_| BrowserCommandError::Internal)?;
    session_store::save_at(&directory, session_id, &key, &model)
        .map_err(|_| BrowserCommandError::Internal)?;
    Ok(model)
}

pub(super) fn save_session(
    session_id: &str,
    model: &SessionModel,
) -> Result<(), BrowserCommandError> {
    let key = session_store::session_key().map_err(|_| BrowserCommandError::Unavailable)?;
    let directory = session_store::sessions_dir();
    session_store::save_at(&directory, session_id, &key, model)
        .map_err(|_| BrowserCommandError::Internal)
}
