use super::{
    browser_view_key::BrowserViewKey, runtime_revision::RuntimeStamp,
    session_model::BrowserSessionState, session_persistence,
    session_service::BrowserSessionService, session_types::BrowserRuntimeTabUpdate,
    BrowserCommandError,
};

impl BrowserSessionService {
    pub(super) fn update_runtime(
        &self,
        session_id: &str,
        tab_id: &str,
        stamp: RuntimeStamp,
        mut update: BrowserRuntimeTabUpdate,
    ) -> Result<Option<BrowserSessionState>, BrowserCommandError> {
        let view_key = validated_view_key(session_id, tab_id)?;
        let _guard = self
            .gate
            .lock()
            .map_err(|_| BrowserCommandError::Internal)?;
        if !self
            .runtime_revisions
            .lock()
            .map_err(|_| BrowserCommandError::Internal)?
            .filter_update(view_key, stamp, &mut update)
        {
            return Ok(None);
        }
        let mut sessions = self
            .live_sessions
            .lock()
            .map_err(|_| BrowserCommandError::Internal)?;
        let model = self.session(&mut sessions, session_id)?;
        let before = model.clone();
        let outcome = match model.update_runtime(tab_id, &update) {
            Ok(outcome) => outcome,
            Err(()) => {
                *model = before;
                return Err(BrowserCommandError::InvalidInput);
            }
        };
        if outcome.persisted_changed {
            if let Err(error) = session_persistence::save_session(session_id, model) {
                *model = before;
                return Err(error);
            }
        }
        Ok(outcome.changed.then(|| model.state().clone()))
    }

    pub(super) fn mark_released(
        &self,
        session_id: &str,
        tab_id: &str,
        stamp: RuntimeStamp,
    ) -> Result<Option<BrowserSessionState>, BrowserCommandError> {
        let view_key = validated_view_key(session_id, tab_id)?;
        let _guard = self
            .gate
            .lock()
            .map_err(|_| BrowserCommandError::Internal)?;
        if !self
            .runtime_revisions
            .lock()
            .map_err(|_| BrowserCommandError::Internal)?
            .accept_release(view_key, stamp)
        {
            return Ok(None);
        }
        let mut sessions = self
            .live_sessions
            .lock()
            .map_err(|_| BrowserCommandError::Internal)?;
        let model = self.session(&mut sessions, session_id)?;
        let before = model.clone();
        let changed = match model.mark_released(tab_id) {
            Ok(changed) => changed,
            Err(()) => {
                *model = before;
                return Err(BrowserCommandError::InvalidInput);
            }
        };
        Ok(changed.then(|| model.state().clone()))
    }
}

fn validated_view_key(
    session_id: &str,
    tab_id: &str,
) -> Result<BrowserViewKey, BrowserCommandError> {
    BrowserViewKey::new(session_id.to_owned(), tab_id.to_owned())
        .map_err(|_| BrowserCommandError::InvalidInput)
}
