use super::{
    live_session_registry::LiveSessionRegistry,
    session_model::{BrowserSessionState, BrowserTabCreation, SessionModel},
    session_persistence,
    tab_id::new_secure_tab_id,
    BrowserCommandError,
};
use std::sync::{Arc, Mutex};

#[derive(Clone, Default)]
pub struct BrowserSessionService {
    pub(super) gate: Arc<Mutex<()>>,
    pub(super) live_sessions: Arc<Mutex<LiveSessionRegistry>>,
    #[cfg(any(target_os = "macos", target_os = "windows"))]
    pub(super) runtime_revisions: Arc<Mutex<super::runtime_revision::RuntimeRevisionCache>>,
}

impl BrowserSessionService {
    pub fn open(&self, session_id: &str) -> Result<BrowserSessionState, BrowserCommandError> {
        validate_session_id(session_id)?;
        let _guard = self
            .gate
            .lock()
            .map_err(|_| BrowserCommandError::Internal)?;
        let mut sessions = self
            .live_sessions
            .lock()
            .map_err(|_| BrowserCommandError::Internal)?;
        Ok(self.session(&mut sessions, session_id)?.state().clone())
    }

    pub fn create_tab(
        &self,
        session_id: &str,
        replacement: Option<&str>,
    ) -> Result<BrowserTabCreation, BrowserCommandError> {
        self.mutate(session_id, |model| {
            model
                .create_tab(new_secure_tab_id(), replacement)
                .map_err(|_| BrowserCommandError::InvalidInput)
        })
    }

    pub fn activate_tab(
        &self,
        session_id: &str,
        tab_id: &str,
    ) -> Result<BrowserSessionState, BrowserCommandError> {
        self.mutate(session_id, |model| {
            model
                .activate_tab(tab_id)
                .map_err(|_| BrowserCommandError::InvalidInput)?;
            Ok(model.state().clone())
        })
    }

    pub fn reorder_tabs(
        &self,
        session_id: &str,
        tab_ids: &[String],
    ) -> Result<BrowserSessionState, BrowserCommandError> {
        self.mutate(session_id, |model| {
            model
                .reorder_tabs(tab_ids)
                .map_err(|_| BrowserCommandError::InvalidInput)?;
            Ok(model.state().clone())
        })
    }

    pub fn close_tab(
        &self,
        session_id: &str,
        tab_id: &str,
    ) -> Result<BrowserSessionState, BrowserCommandError> {
        self.mutate(session_id, |model| {
            model
                .close_tab(tab_id, new_secure_tab_id())
                .map_err(|_| BrowserCommandError::InvalidInput)?;
            Ok(model.state().clone())
        })
    }

    pub fn navigate(
        &self,
        session_id: &str,
        tab_id: &str,
        url: &str,
    ) -> Result<BrowserSessionState, BrowserCommandError> {
        self.mutate(session_id, |model| {
            model
                .navigate(tab_id, url)
                .map_err(|_| BrowserCommandError::InvalidInput)?;
            Ok(model.state().clone())
        })
    }

    fn mutate<T>(
        &self,
        session_id: &str,
        operation: impl FnOnce(&mut SessionModel) -> Result<T, BrowserCommandError>,
    ) -> Result<T, BrowserCommandError> {
        validate_session_id(session_id)?;
        let _guard = self
            .gate
            .lock()
            .map_err(|_| BrowserCommandError::Internal)?;
        let mut sessions = self
            .live_sessions
            .lock()
            .map_err(|_| BrowserCommandError::Internal)?;
        let model = self.session(&mut sessions, session_id)?;
        let before = model.clone();
        let persisted_before = model.persisted();
        let result = match operation(model) {
            Ok(result) => result,
            Err(error) => {
                *model = before;
                return Err(error);
            }
        };
        if model.persisted() != persisted_before {
            if let Err(error) = session_persistence::save_session(session_id, model) {
                *model = before;
                return Err(error);
            }
        }
        Ok(result)
    }

    pub(super) fn session<'a>(
        &self,
        sessions: &'a mut LiveSessionRegistry,
        session_id: &str,
    ) -> Result<&'a mut SessionModel, BrowserCommandError> {
        if !sessions.contains(session_id) {
            let model = session_persistence::load_or_create(session_id)?;
            sessions.insert(session_id.to_owned(), model);
        }
        sessions
            .get_mut(session_id)
            .ok_or(BrowserCommandError::Internal)
    }
}

fn validate_session_id(session_id: &str) -> Result<(), BrowserCommandError> {
    crate::services::agent_local::session_store::validate_session_id(session_id)
        .map_err(|_| BrowserCommandError::InvalidInput)
}
