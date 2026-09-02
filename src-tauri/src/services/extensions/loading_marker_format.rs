use serde::{Deserialize, Serialize};

const VERSION: u8 = 1;
const MAX_ATTEMPTS: u8 = 3;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct LoadingMarker {
    version: u8,
    pub(crate) extension_id: String,
    pub(crate) stage: String,
    started_at: String,
    pub(crate) attempts: u8,
}

impl LoadingMarker {
    pub(super) fn new(extension_id: &str, attempts: u8) -> Result<Self, String> {
        super::validation::identifier(extension_id).map_err(|_| invalid())?;
        if !(1..=MAX_ATTEMPTS).contains(&attempts) {
            return Err(invalid());
        }
        Ok(Self {
            version: VERSION,
            extension_id: extension_id.to_string(),
            stage: super::types::HOST_LOAD_STAGES[0].to_string(),
            started_at: chrono::Utc::now().to_rfc3339(),
            attempts,
        })
    }

    pub(crate) fn can_retry(&self) -> bool {
        self.attempts < MAX_ATTEMPTS
    }

    pub(super) fn valid(&self) -> bool {
        self.version == VERSION
            && super::validation::identifier(&self.extension_id).is_ok()
            && super::types::HOST_LOAD_STAGES.contains(&self.stage.as_str())
            && (1..=MAX_ATTEMPTS).contains(&self.attempts)
            && chrono::DateTime::parse_from_rfc3339(&self.started_at).is_ok()
    }
}

pub(super) fn parse(bytes: &[u8]) -> Option<LoadingMarker> {
    serde_json::from_slice::<LoadingMarker>(bytes)
        .ok()
        .filter(LoadingMarker::valid)
}

pub(super) fn serialize(marker: &LoadingMarker) -> Result<Vec<u8>, String> {
    serde_json::to_vec(marker).map_err(|_| invalid())
}

fn invalid() -> String {
    super::error_codes::RECOVERY_MARKER_INVALID.to_string()
}
