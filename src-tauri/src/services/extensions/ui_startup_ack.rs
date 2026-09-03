use std::path::PathBuf;
use std::sync::Mutex;

use rand::RngCore;
use subtle::ConstantTimeEq;
use zeroize::{Zeroize, Zeroizing};

use super::loading_marker;

pub(crate) const ACK_TOKEN_BYTES: usize = 32;
pub(crate) type UiAckToken = [u8; ACK_TOKEN_BYTES];

struct ActiveAttempt {
    extension_id: String,
    token: Zeroizing<UiAckToken>,
}

pub(crate) struct UiLoadAcknowledger {
    marker_path: PathBuf,
    active: Mutex<Option<ActiveAttempt>>,
}

impl UiLoadAcknowledger {
    pub(crate) fn new() -> Self {
        Self::for_path(loading_marker::path())
    }

    pub(crate) fn for_path(marker_path: PathBuf) -> Self {
        Self {
            marker_path,
            active: Mutex::new(None),
        }
    }

    pub(crate) fn begin(&self, extension_id: &str, attempts: u8) -> Result<UiAckToken, String> {
        self.begin_with_fill(extension_id, attempts, |token| {
            rand::rngs::OsRng
                .try_fill_bytes(token)
                .map_err(|_| invalid())
        })
    }

    fn begin_with_fill(
        &self,
        extension_id: &str,
        attempts: u8,
        fill: impl FnOnce(&mut [u8]) -> Result<(), String>,
    ) -> Result<UiAckToken, String> {
        let mut active = self.active.lock().map_err(|_| invalid())?;
        if active.is_some() {
            return Err(invalid());
        }
        if self.marker_path == loading_marker::path() {
            loading_marker::ui_start(extension_id, attempts)?;
        } else {
            loading_marker::ui_start_at(&self.marker_path, extension_id, attempts)?;
        }
        let mut token = Zeroizing::new([0_u8; ACK_TOKEN_BYTES]);
        fill(token.as_mut())?;
        let projection = *token;
        *active = Some(ActiveAttempt {
            extension_id: extension_id.to_string(),
            token,
        });
        Ok(projection)
    }

    #[cfg(test)]
    pub(crate) fn begin_rng_failure_for_test(
        &self,
        extension_id: &str,
        attempts: u8,
    ) -> Result<UiAckToken, String> {
        self.begin_with_fill(extension_id, attempts, |_| Err(invalid()))
    }

    pub(crate) fn acknowledge(
        &self,
        extension_id: &str,
        candidate: &UiAckToken,
    ) -> Result<(), String> {
        let mut active = self.active.lock().map_err(|_| invalid())?;
        let Some(attempt) = active.as_ref() else {
            return Err(invalid());
        };
        let token_matches = attempt.token.as_ref().ct_eq(candidate.as_slice());
        let identity_matches = attempt.extension_id == extension_id;
        if !identity_matches || !bool::from(token_matches) {
            return Err(invalid());
        }
        if self.marker_path == loading_marker::path() {
            loading_marker::ui_complete(extension_id)?;
        } else {
            loading_marker::ui_complete_at(&self.marker_path, extension_id)?;
        }
        if let Some(mut completed) = active.take() {
            completed.token.zeroize();
        }
        Ok(())
    }

    pub(crate) fn abort(&self, extension_id: &str, candidate: &UiAckToken) -> Result<(), String> {
        self.acknowledge(extension_id, candidate)
    }

    pub(crate) fn advance(
        &self,
        extension_id: &str,
        candidate: &UiAckToken,
        stage: &str,
    ) -> Result<(), String> {
        let active = self.active.lock().map_err(|_| invalid())?;
        let Some(attempt) = active.as_ref() else {
            return Err(invalid());
        };
        let token_matches = attempt.token.as_ref().ct_eq(candidate.as_slice());
        if attempt.extension_id != extension_id || !bool::from(token_matches) {
            return Err(invalid());
        }
        if self.marker_path == loading_marker::path() {
            loading_marker::ui_advance(extension_id, stage)
        } else {
            loading_marker::ui_advance_at(&self.marker_path, extension_id, stage)
        }
    }

    #[cfg(test)]
    pub(crate) fn fail_active_attempt(&self) {
        if let Ok(mut active) = self.active.lock() {
            if let Some(mut failed) = active.take() {
                failed.token.zeroize();
            }
        }
    }
}

fn invalid() -> String {
    super::error_codes::RECOVERY_MARKER_INVALID.to_string()
}
