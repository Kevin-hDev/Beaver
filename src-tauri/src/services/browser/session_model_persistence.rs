use super::{
    session_model::{BrowserSessionState, BrowserTabState, SessionModel},
    session_types::{
        PersistedBrowserSession, PersistedBrowserSessionState, PersistedBrowserTabState,
        SESSION_VERSION,
    },
    session_validation::validate_persisted,
};
use serde::Deserialize;

impl SessionModel {
    pub(super) fn restore(bytes: &[u8]) -> Result<Self, ()> {
        let version: PersistedVersion = serde_json::from_slice(bytes).map_err(|_| ())?;
        let persisted = match version.version {
            1 => {
                let legacy: LegacyPersistedBrowserSession =
                    serde_json::from_slice(bytes).map_err(|_| ())?;
                PersistedBrowserSession {
                    version: SESSION_VERSION,
                    state: persisted_state(&legacy.state),
                    recency: legacy.recency,
                }
            }
            SESSION_VERSION => serde_json::from_slice(bytes).map_err(|_| ())?,
            _ => return Err(()),
        };
        validate_persisted(&persisted)?;
        let tabs = persisted
            .state
            .tabs
            .into_iter()
            .map(|tab| BrowserTabState {
                released: tab.url.is_some(),
                id: tab.id,
                title: tab.title,
                url: tab.url,
                loading: false,
                can_go_back: false,
                can_go_forward: false,
            })
            .collect();
        Ok(Self {
            state: BrowserSessionState {
                tabs,
                active_tab_id: persisted.state.active_tab_id,
                generation: persisted.state.generation,
            },
            recency: persisted.recency.into(),
        })
    }

    pub(super) fn persisted(&self) -> PersistedBrowserSession {
        PersistedBrowserSession {
            version: SESSION_VERSION,
            state: persisted_state(&self.state),
            recency: self.recency.iter().cloned().collect(),
        }
    }
}

#[derive(Deserialize)]
struct PersistedVersion {
    version: u8,
}

#[derive(Deserialize)]
struct LegacyPersistedBrowserSession {
    state: BrowserSessionState,
    recency: Vec<String>,
}

fn persisted_state(state: &BrowserSessionState) -> PersistedBrowserSessionState {
    PersistedBrowserSessionState {
        tabs: state
            .tabs
            .iter()
            .map(|tab| PersistedBrowserTabState {
                id: tab.id.clone(),
                title: tab.title.clone(),
                url: tab.url.clone(),
            })
            .collect(),
        active_tab_id: state.active_tab_id.clone(),
        generation: state.generation,
    }
}
