#[cfg(any(target_os = "macos", target_os = "windows"))]
use super::{
    browser_view_key::BrowserViewKey,
    runtime_revision::{RuntimeRevisionCache, RuntimeStamp},
    session_types::BrowserRuntimeTabUpdate,
};
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
    gate: Arc<Mutex<()>>,
    live_sessions: Arc<Mutex<LiveSessionRegistry>>,
    #[cfg(any(target_os = "macos", target_os = "windows"))]
    runtime_revisions: Arc<Mutex<RuntimeRevisionCache>>,
}

impl BrowserSessionService {
    pub fn open(&self, session_id: &str) -> Result<BrowserSessionState, BrowserCommandError> {
        validate_session_id(session_id)?;
        let _guard = self
            .gate
            .lock()
            .map_err(|_| BrowserCommandError::Internal)?;
        let cold = self.activate_session(session_id)?;
        session_persistence::open_session(session_id, cold)
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

    #[cfg(any(target_os = "macos", target_os = "windows"))]
    pub(super) fn update_runtime(
        &self,
        session_id: &str,
        tab_id: &str,
        stamp: RuntimeStamp,
        update: BrowserRuntimeTabUpdate,
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
            .accept_update(view_key, stamp)
        {
            return Ok(None);
        }
        let cold = self.activate_session(session_id)?;
        session_persistence::mutate_session(session_id, cold, |model| {
            let changed = model
                .update_runtime(tab_id, &update)
                .map_err(|_| BrowserCommandError::InvalidInput)?;
            Ok(changed.then(|| model.state().clone()))
        })
    }

    #[cfg(any(target_os = "macos", target_os = "windows"))]
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
        let cold = self.activate_session(session_id)?;
        session_persistence::mutate_session(session_id, cold, |model| {
            let changed = model
                .mark_released(tab_id)
                .map_err(|_| BrowserCommandError::InvalidInput)?;
            Ok(changed.then(|| model.state().clone()))
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
        let cold = self.activate_session(session_id)?;
        session_persistence::mutate_session(session_id, cold, operation)
    }

    fn activate_session(&self, session_id: &str) -> Result<bool, BrowserCommandError> {
        self.live_sessions
            .lock()
            .map(|mut registry| registry.activate(session_id))
            .map_err(|_| BrowserCommandError::Internal)
    }
}

fn validate_session_id(session_id: &str) -> Result<(), BrowserCommandError> {
    crate::services::agent_local::session_store::validate_session_id(session_id)
        .map_err(|_| BrowserCommandError::InvalidInput)
}

#[cfg(any(target_os = "macos", target_os = "windows"))]
fn validated_view_key(
    session_id: &str,
    tab_id: &str,
) -> Result<BrowserViewKey, BrowserCommandError> {
    BrowserViewKey::new(session_id.to_owned(), tab_id.to_owned())
        .map_err(|_| BrowserCommandError::InvalidInput)
}
