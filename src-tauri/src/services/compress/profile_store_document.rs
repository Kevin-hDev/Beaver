use serde::{Deserialize, Serialize};

use super::profile_defaults::{beaver_profile, BEAVER_PROFILE_ID};
use super::profile_limits::MAX_PROFILES;
use super::profile_types::CompressionProfile;
use super::profile_validation::{normalize_profile_document, validate_profile_input};

pub const PROFILE_SCHEMA_VERSION: u16 = 2;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct CompressionProfileDocument {
    pub schema_version: u16,
    pub recovery_backup_pending: bool,
    pub automatic_enabled: bool,
    pub global_profile_id: String,
    pub global_selection_revision: u64,
    pub profiles: Vec<CompressionProfile>,
}

impl Default for CompressionProfileDocument {
    fn default() -> Self {
        Self {
            schema_version: PROFILE_SCHEMA_VERSION,
            recovery_backup_pending: false,
            automatic_enabled: true,
            global_profile_id: BEAVER_PROFILE_ID.to_string(),
            global_selection_revision: 1,
            profiles: vec![beaver_profile()],
        }
    }
}

impl CompressionProfileDocument {
    pub fn normalize(&mut self) {
        normalize_profile_document(&mut self.profiles, &mut self.global_profile_id);
        self.global_selection_revision = self.global_selection_revision.max(1);
        for profile in &mut self.profiles {
            profile.revision = profile.revision.max(1);
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.schema_version != PROFILE_SCHEMA_VERSION
            || self.profiles.is_empty()
            || self.profiles.len() > MAX_PROFILES
            || !self
                .profiles
                .iter()
                .any(|profile| profile.id == BEAVER_PROFILE_ID)
            || !self
                .profiles
                .iter()
                .any(|profile| profile.id == self.global_profile_id)
        {
            return Err("invalid compression profile document".to_string());
        }
        for profile in &self.profiles {
            validate_profile_input(profile, &self.profiles)
                .map_err(|_| "invalid compression profile document".to_string())?;
        }
        Ok(())
    }
}
