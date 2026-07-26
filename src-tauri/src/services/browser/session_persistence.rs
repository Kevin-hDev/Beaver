use super::{
    session_model::{BrowserSessionState, SessionModel},
    session_store,
    tab_id::new_secure_tab_id,
    BrowserCommandError,
};

pub(super) fn open_session(
    session_id: &str,
    cold: bool,
) -> Result<BrowserSessionState, BrowserCommandError> {
    let key = session_store::session_key().map_err(|_| BrowserCommandError::Unavailable)?;
    let directory = session_store::sessions_dir();
    if let Some(mut model) = session_store::load_at(&directory, session_id, &key)
        .map_err(|_| BrowserCommandError::Internal)?
    {
        if cold
            && model
                .release_runtime()
                .map_err(|_| BrowserCommandError::Internal)?
        {
            session_store::save_at(&directory, session_id, &key, &model)
                .map_err(|_| BrowserCommandError::Internal)?;
        }
        return Ok(model.state().clone());
    }
    let model =
        SessionModel::new(new_secure_tab_id()).map_err(|_| BrowserCommandError::Internal)?;
    session_store::save_at(&directory, session_id, &key, &model)
        .map_err(|_| BrowserCommandError::Internal)?;
    Ok(model.state().clone())
}

pub(super) fn mutate_session<T>(
    session_id: &str,
    cold: bool,
    operation: impl FnOnce(&mut SessionModel) -> Result<T, BrowserCommandError>,
) -> Result<T, BrowserCommandError> {
    let key = session_store::session_key().map_err(|_| BrowserCommandError::Unavailable)?;
    let directory = session_store::sessions_dir();
    let loaded = session_store::load_at(&directory, session_id, &key)
        .map_err(|_| BrowserCommandError::Internal)?;
    let is_new = loaded.is_none();
    let mut model = match loaded {
        Some(model) => model,
        None => {
            SessionModel::new(new_secure_tab_id()).map_err(|_| BrowserCommandError::Internal)?
        }
    };
    let previous_generation = model.state().generation;
    if cold {
        model
            .release_runtime()
            .map_err(|_| BrowserCommandError::Internal)?;
    }
    let result = operation(&mut model)?;
    if is_new || model.state().generation != previous_generation {
        session_store::save_at(&directory, session_id, &key, &model)
            .map_err(|_| BrowserCommandError::Internal)?;
    }
    Ok(result)
}
