use serde::{Deserialize, Serialize};

use super::loading_marker_format::LoadingMarker;

const VERSION: u8 = 2;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct LoadingJournal {
    version: u8,
    #[serde(skip_serializing_if = "Option::is_none")]
    host: Option<LoadingMarker>,
    #[serde(skip_serializing_if = "Option::is_none")]
    ui: Option<LoadingMarker>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct LegacyMarker {
    version: u8,
    extension_id: String,
    stage: String,
    started_at: String,
    attempts: u8,
}

impl LoadingJournal {
    pub(super) fn empty() -> Self {
        Self {
            version: VERSION,
            host: None,
            ui: None,
        }
    }

    #[cfg(test)]
    pub(crate) fn version(&self) -> u8 {
        self.version
    }

    pub(crate) fn host(&self) -> Option<&LoadingMarker> {
        self.host.as_ref()
    }

    pub(crate) fn ui(&self) -> Option<&LoadingMarker> {
        self.ui.as_ref()
    }

    pub(super) fn host_mut(&mut self) -> &mut Option<LoadingMarker> {
        &mut self.host
    }

    pub(super) fn ui_mut(&mut self) -> &mut Option<LoadingMarker> {
        &mut self.ui
    }

    pub(super) fn is_empty(&self) -> bool {
        self.host.is_none() && self.ui.is_none()
    }

    fn valid(&self) -> bool {
        self.version == VERSION
            && !self.is_empty()
            && self.host.as_ref().is_none_or(LoadingMarker::valid_host)
            && self.ui.as_ref().is_none_or(LoadingMarker::valid_ui)
    }
}

pub(super) fn parse(bytes: &[u8]) -> Option<LoadingJournal> {
    // This journal gates safe startup. Ambiguous persisted bytes must therefore
    // fail closed instead of using the application's usual tolerant config path.
    if let Ok(journal) = serde_json::from_slice::<LoadingJournal>(bytes) {
        return journal.valid().then_some(journal);
    }
    let legacy = serde_json::from_slice::<LegacyMarker>(bytes).ok()?;
    if legacy.version != 1 {
        return None;
    }
    let marker = LoadingMarker {
        extension_id: legacy.extension_id,
        stage: legacy.stage,
        started_at: legacy.started_at,
        attempts: legacy.attempts,
    };
    marker.valid_host().then_some(LoadingJournal {
        version: VERSION,
        host: Some(marker),
        ui: None,
    })
}

pub(super) fn serialize(journal: &LoadingJournal) -> Result<Vec<u8>, String> {
    if !journal.valid() {
        return Err(invalid());
    }
    serde_json::to_vec(journal).map_err(|_| invalid())
}

fn invalid() -> String {
    super::error_codes::RECOVERY_MARKER_INVALID.to_string()
}
