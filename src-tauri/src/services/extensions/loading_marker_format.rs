use serde::{Deserialize, Serialize};

pub(super) const MAX_ATTEMPTS: u8 = 3;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct LoadingMarker {
    pub(crate) extension_id: String,
    pub(crate) stage: String,
    pub(crate) started_at: String,
    pub(crate) attempts: u8,
}

impl LoadingMarker {
    pub(super) fn new_host(extension_id: &str, attempts: u8) -> Result<Self, String> {
        Self::new(extension_id, super::types::HOST_LOAD_STAGES[0], attempts)
    }

    pub(super) fn new_ui(extension_id: &str, attempts: u8) -> Result<Self, String> {
        Self::new(
            extension_id,
            super::ui_contract::UI_LOADING_STAGES[0],
            attempts,
        )
    }

    fn new(extension_id: &str, stage: &str, attempts: u8) -> Result<Self, String> {
        super::validation::identifier(extension_id).map_err(|_| invalid())?;
        if !(1..=MAX_ATTEMPTS).contains(&attempts) {
            return Err(invalid());
        }
        Ok(Self {
            extension_id: extension_id.to_string(),
            stage: stage.to_string(),
            started_at: chrono::Utc::now().to_rfc3339(),
            attempts,
        })
    }

    pub(crate) fn can_retry(&self) -> bool {
        self.attempts < MAX_ATTEMPTS
    }

    pub(super) fn valid_host(&self) -> bool {
        self.valid(super::types::HOST_LOAD_STAGES)
    }

    pub(super) fn valid_ui(&self) -> bool {
        self.valid(super::ui_contract::UI_LOADING_STAGES)
    }

    fn valid(&self, stages: &[&str]) -> bool {
        super::validation::identifier(&self.extension_id).is_ok()
            && stages.contains(&self.stage.as_str())
            && (1..=MAX_ATTEMPTS).contains(&self.attempts)
            && chrono::DateTime::parse_from_rfc3339(&self.started_at).is_ok()
    }
}

fn invalid() -> String {
    super::error_codes::RECOVERY_MARKER_INVALID.to_string()
}
