use super::{
    session_types::{blank_tab, PersistedBrowserSession, SESSION_VERSION},
    session_validation::validate_persisted,
    tab_id::validate_tab_id,
    url_policy::validate_browser_url,
};
use std::collections::VecDeque;

pub use super::session_types::{
    BrowserSessionState, BrowserTabCreation, BrowserTabState, MAX_BROWSER_TABS,
};

pub(super) struct SessionModel {
    state: BrowserSessionState,
    recency: VecDeque<String>,
}

impl SessionModel {
    pub(super) fn new(tab_id: String) -> Result<Self, ()> {
        validate_tab_id(&tab_id)?;
        Ok(Self {
            state: BrowserSessionState {
                tabs: vec![blank_tab(tab_id.clone())],
                active_tab_id: tab_id.clone(),
                generation: 1,
            },
            recency: VecDeque::from([tab_id]),
        })
    }

    pub(super) fn restore(bytes: &[u8]) -> Result<Self, ()> {
        let persisted: PersistedBrowserSession = serde_json::from_slice(bytes).map_err(|_| ())?;
        validate_persisted(&persisted)?;
        Ok(Self {
            state: persisted.state,
            recency: persisted.recency.into(),
        })
    }

    pub(super) fn release_runtime(&mut self) -> Result<bool, ()> {
        let mut changed = false;
        for tab in &mut self.state.tabs {
            let released = tab.url.is_some();
            changed |=
                tab.loading || tab.can_go_back || tab.can_go_forward || tab.released != released;
            tab.loading = false;
            tab.can_go_back = false;
            tab.can_go_forward = false;
            tab.released = released;
        }
        if changed {
            self.bump()?;
        }
        Ok(changed)
    }

    pub(super) fn state(&self) -> &BrowserSessionState {
        &self.state
    }

    pub(super) fn persisted(&self) -> PersistedBrowserSession {
        PersistedBrowserSession {
            version: SESSION_VERSION,
            state: self.state.clone(),
            recency: self.recency.iter().cloned().collect(),
        }
    }

    pub(super) fn create_tab(
        &mut self,
        new_id: String,
        replacement: Option<&str>,
    ) -> Result<BrowserTabCreation, ()> {
        validate_tab_id(&new_id)?;
        if self.state.tabs.iter().any(|tab| tab.id == new_id) {
            return Err(());
        }
        if self.state.tabs.len() < MAX_BROWSER_TABS {
            if replacement.is_some() {
                return Err(());
            }
            self.state.tabs.push(blank_tab(new_id.clone()));
        } else {
            let candidate = self.oldest_inactive().ok_or(())?.to_owned();
            if replacement.is_none() {
                return Ok(BrowserTabCreation::ConfirmationRequired {
                    candidate_title: self.tab(&candidate)?.title.clone(),
                    candidate_id: candidate,
                });
            }
            if replacement != Some(candidate.as_str()) {
                return Err(());
            }
            let index = self
                .state
                .tabs
                .iter()
                .position(|tab| tab.id == candidate)
                .ok_or(())?;
            self.state.tabs[index] = blank_tab(new_id.clone());
            self.remove_recency(&candidate);
        }
        self.state.active_tab_id = new_id.clone();
        self.touch(new_id);
        self.bump()?;
        Ok(BrowserTabCreation::Created {
            session: self.state.clone(),
        })
    }

    pub(super) fn activate_tab(&mut self, id: &str) -> Result<(), ()> {
        self.tab(id)?;
        self.state.active_tab_id = id.to_owned();
        self.touch(id.to_owned());
        self.bump()
    }

    pub(super) fn close_tab(&mut self, id: &str, new_id: String) -> Result<(), ()> {
        let index = self
            .state
            .tabs
            .iter()
            .position(|tab| tab.id == id)
            .ok_or(())?;
        self.state.tabs.remove(index);
        self.remove_recency(id);
        if self.state.tabs.is_empty() {
            validate_tab_id(&new_id)?;
            self.state.tabs.push(blank_tab(new_id.clone()));
            self.state.active_tab_id = new_id.clone();
            self.touch(new_id);
        } else if self.state.active_tab_id == id {
            let next = index.min(self.state.tabs.len() - 1);
            let active_id = self.state.tabs[next].id.clone();
            self.state.active_tab_id.clone_from(&active_id);
            self.touch(active_id);
        }
        self.bump()
    }

    pub(super) fn navigate(&mut self, id: &str, raw_url: &str) -> Result<(), ()> {
        let url = validate_browser_url(raw_url)?;
        let tab = self.tab_mut(id)?;
        tab.url = Some(url.as_str().to_owned());
        tab.loading = true;
        tab.released = false;
        self.bump()
    }

    fn oldest_inactive(&self) -> Option<&str> {
        self.recency
            .iter()
            .find(|id| id.as_str() != self.state.active_tab_id)
            .map(String::as_str)
    }

    fn tab(&self, id: &str) -> Result<&BrowserTabState, ()> {
        self.state.tabs.iter().find(|tab| tab.id == id).ok_or(())
    }

    pub(super) fn tab_mut(&mut self, id: &str) -> Result<&mut BrowserTabState, ()> {
        self.state
            .tabs
            .iter_mut()
            .find(|tab| tab.id == id)
            .ok_or(())
    }

    fn touch(&mut self, id: String) {
        self.remove_recency(&id);
        self.recency.push_back(id);
    }

    fn remove_recency(&mut self, id: &str) {
        self.recency.retain(|entry| entry != id);
    }

    pub(super) fn bump(&mut self) -> Result<(), ()> {
        self.state.generation = self.state.generation.checked_add(1).ok_or(())?;
        Ok(())
    }
}
