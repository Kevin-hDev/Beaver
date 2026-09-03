use std::sync::{Arc, Mutex};

use serde::Serialize;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) enum SafeReason {
    Argument,
    Shift,
    InvalidMarker,
    RecoveryChoice,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub(crate) enum UiStartupMode {
    Normal,
    Safe {
        reason: SafeReason,
    },
    PendingInterruptedUi {
        extension_id: String,
        stage: String,
        started_at: String,
        attempts: u8,
    },
    RetryInterruptedUi {
        extension_id: String,
        attempts: u8,
    },
    #[allow(dead_code, reason = "constructed only by Linux Wayland startup builds")]
    AwaitingWayland,
}

#[derive(Clone, Debug)]
pub(crate) struct UiStartupState {
    resolution: Arc<Mutex<StartupResolution>>,
}

#[derive(Debug)]
struct StartupResolution {
    mode: UiStartupMode,
    wayland_fallback: Option<UiStartupMode>,
}

impl UiStartupState {
    pub(crate) fn resolved(mode: UiStartupMode) -> Self {
        Self {
            resolution: Arc::new(Mutex::new(StartupResolution {
                mode,
                wayland_fallback: None,
            })),
        }
    }

    pub(super) fn awaiting_wayland(fallback: UiStartupMode) -> Self {
        Self {
            resolution: Arc::new(Mutex::new(StartupResolution {
                mode: UiStartupMode::AwaitingWayland,
                wayland_fallback: Some(fallback),
            })),
        }
    }

    pub(crate) fn mode(&self) -> UiStartupMode {
        self.resolution.lock().map_or(
            UiStartupMode::Safe {
                reason: SafeReason::InvalidMarker,
            },
            |resolution| resolution.mode.clone(),
        )
    }

    pub(crate) fn bootstrap_resolved(&self) -> bool {
        !matches!(self.mode(), UiStartupMode::AwaitingWayland)
    }

    pub(crate) fn third_party_loading_allowed(&self) -> bool {
        matches!(self.mode(), UiStartupMode::Normal)
    }

    pub(crate) fn loading_allowed_for(&self, extension_id: &str, attempts: u8) -> bool {
        match self.mode() {
            UiStartupMode::Normal => true,
            UiStartupMode::RetryInterruptedUi {
                extension_id: target,
                attempts: target_attempts,
            } => target == extension_id && target_attempts == attempts,
            _ => false,
        }
    }

    pub(crate) fn protocol_loading_allowed_for(&self, extension_id: &str) -> bool {
        match self.mode() {
            UiStartupMode::Normal => true,
            UiStartupMode::RetryInterruptedUi {
                extension_id: target,
                ..
            } => target == extension_id,
            _ => false,
        }
    }

    pub(crate) fn confirm_wayland_shift(&self, shift: bool) -> Result<(), String> {
        let mut resolution = self.resolution.lock().map_err(|_| invalid())?;
        if !matches!(resolution.mode, UiStartupMode::AwaitingWayland) {
            return Err(invalid());
        }
        resolution.mode = if shift {
            UiStartupMode::Safe {
                reason: SafeReason::Shift,
            }
        } else {
            resolution.wayland_fallback.take().ok_or_else(invalid)?
        };
        resolution.wayland_fallback = None;
        Ok(())
    }

    pub(crate) fn choose_safe(&self) -> Result<(), String> {
        let mut resolution = self.resolution.lock().map_err(|_| invalid())?;
        match resolution.mode {
            UiStartupMode::PendingInterruptedUi { .. }
            | UiStartupMode::Safe {
                reason: SafeReason::InvalidMarker,
            } => {}
            _ => return Err(invalid()),
        }
        resolution.mode = UiStartupMode::Safe {
            reason: SafeReason::RecoveryChoice,
        };
        Ok(())
    }

    pub(crate) fn retry_pending(&self) -> Result<(), String> {
        let mut resolution = self.resolution.lock().map_err(|_| invalid())?;
        let UiStartupMode::PendingInterruptedUi {
            ref extension_id,
            attempts,
            ..
        } = resolution.mode
        else {
            return Err(invalid());
        };
        if attempts >= super::loading_marker_format::MAX_ATTEMPTS {
            return Err(invalid());
        }
        resolution.mode = UiStartupMode::RetryInterruptedUi {
            extension_id: extension_id.clone(),
            attempts: attempts + 1,
        };
        Ok(())
    }

    pub(crate) fn complete_authorized_load(&self, extension_id: &str) -> Result<(), String> {
        let mut resolution = self.resolution.lock().map_err(|_| invalid())?;
        if let UiStartupMode::RetryInterruptedUi {
            extension_id: target,
            ..
        } = &resolution.mode
        {
            if target != extension_id {
                return Err(invalid());
            }
            resolution.mode = UiStartupMode::Normal;
        }
        Ok(())
    }

    pub(crate) fn acknowledge_invalid_marker(&self) -> Result<(), String> {
        let mut resolution = self.resolution.lock().map_err(|_| invalid())?;
        if !matches!(
            resolution.mode,
            UiStartupMode::Safe {
                reason: SafeReason::InvalidMarker
            }
        ) {
            return Err(invalid());
        }
        resolution.mode = UiStartupMode::Safe {
            reason: SafeReason::RecoveryChoice,
        };
        Ok(())
    }
}

fn invalid() -> String {
    super::error_codes::RECOVERY_MARKER_INVALID.to_string()
}
