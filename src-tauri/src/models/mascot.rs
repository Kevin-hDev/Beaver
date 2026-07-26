use serde::{Deserialize, Serialize};

pub const DEFAULT_MASCOT_ID: &str = "cl-go-beaver";
pub const DEFAULT_SIZE_PERCENT: u16 = 100;
pub const MIN_SIZE_PERCENT: u16 = 70;
pub const MAX_SIZE_PERCENT: u16 = 140;
const POSITION_LIMIT: i32 = 100_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct MascotPosition {
    pub x: i32,
    pub y: i32,
}

impl MascotPosition {
    pub fn normalized(self) -> Option<Self> {
        (self.x.abs() <= POSITION_LIMIT && self.y.abs() <= POSITION_LIMIT).then_some(self)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct MascotSettings {
    pub enabled: bool,
    pub mascot_id: String,
    pub size_percent: u16,
    pub position: Option<MascotPosition>,
}

impl Default for MascotSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            mascot_id: DEFAULT_MASCOT_ID.to_string(),
            size_percent: DEFAULT_SIZE_PERCENT,
            position: None,
        }
    }
}

impl MascotSettings {
    pub fn normalized(mut self) -> Self {
        if self.mascot_id != DEFAULT_MASCOT_ID {
            self.mascot_id = DEFAULT_MASCOT_ID.to_string();
        }
        self.size_percent = self.size_percent.clamp(MIN_SIZE_PERCENT, MAX_SIZE_PERCENT);
        self.position = self.position.and_then(MascotPosition::normalized);
        self
    }
}

#[derive(Debug, Default, Deserialize)]
pub struct MascotSettingsPatch {
    pub enabled: Option<bool>,
    pub mascot_id: Option<String>,
    pub size_percent: Option<u16>,
}

impl MascotSettingsPatch {
    pub fn apply(self, mut current: MascotSettings) -> MascotSettings {
        if let Some(enabled) = self.enabled {
            current.enabled = enabled;
        }
        if let Some(mascot_id) = self.mascot_id {
            current.mascot_id = mascot_id;
        }
        if let Some(size_percent) = self.size_percent {
            current.size_percent = size_percent;
        }
        current.normalized()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn settings_are_bounded_and_unknown_mascots_are_rejected() {
        let settings = MascotSettings {
            mascot_id: "unknown".into(),
            size_percent: 500,
            ..Default::default()
        }
        .normalized();

        assert_eq!(settings.mascot_id, DEFAULT_MASCOT_ID);
        assert_eq!(settings.size_percent, MAX_SIZE_PERCENT);
    }

    #[test]
    fn unsafe_positions_are_discarded() {
        let settings = MascotSettings {
            position: Some(MascotPosition { x: 100_001, y: 0 }),
            ..Default::default()
        }
        .normalized();

        assert_eq!(settings.position, None);
    }
}
