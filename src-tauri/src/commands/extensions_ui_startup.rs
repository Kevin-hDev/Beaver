use serde::Serialize;

use crate::services::extensions::{
    self, SafeReason, UiAckToken, UiLoadAcknowledger, UiStartupMode, UiStartupState,
};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtensionUiStartupProjection {
    mode: UiStartupMode,
    bootstrap_resolved: bool,
    third_party_loading_allowed: bool,
    show_recovery_dialog: bool,
    show_safe_banner: bool,
    can_retry: bool,
}

pub(crate) fn project(state: &UiStartupState) -> ExtensionUiStartupProjection {
    let mode = state.mode();
    let show_recovery_dialog = matches!(
        mode,
        UiStartupMode::PendingInterruptedUi { .. }
            | UiStartupMode::Safe {
                reason: SafeReason::InvalidMarker
            }
    );
    let can_retry = matches!(
        mode,
        UiStartupMode::PendingInterruptedUi { attempts, .. } if attempts < 3
    );
    ExtensionUiStartupProjection {
        bootstrap_resolved: state.bootstrap_resolved(),
        third_party_loading_allowed: state.third_party_loading_allowed(),
        show_safe_banner: matches!(mode, UiStartupMode::Safe { .. }),
        show_recovery_dialog,
        can_retry,
        mode,
    }
}

#[tauri::command]
pub fn get_extension_ui_startup_state(
    state: tauri::State<'_, UiStartupState>,
) -> ExtensionUiStartupProjection {
    project(state.inner())
}

#[tauri::command]
pub fn confirm_extension_ui_wayland_shift(
    state: tauri::State<'_, UiStartupState>,
    shift_pressed: bool,
) -> Result<ExtensionUiStartupProjection, String> {
    state.confirm_wayland_shift(shift_pressed)?;
    Ok(project(state.inner()))
}

#[tauri::command]
pub fn continue_without_extension_ui(
    state: tauri::State<'_, UiStartupState>,
) -> Result<ExtensionUiStartupProjection, String> {
    continue_safe(state.inner())
}

pub(crate) fn continue_safe(
    state: &UiStartupState,
) -> Result<ExtensionUiStartupProjection, String> {
    state.choose_safe()?;
    Ok(project(state))
}

#[tauri::command]
pub fn retry_interrupted_extension_ui(
    state: tauri::State<'_, UiStartupState>,
) -> Result<ExtensionUiStartupProjection, String> {
    state.retry_pending()?;
    Ok(project(state.inner()))
}

#[tauri::command]
pub fn discard_invalid_extension_ui_marker(
    state: tauri::State<'_, UiStartupState>,
) -> Result<ExtensionUiStartupProjection, String> {
    if !matches!(
        state.mode(),
        UiStartupMode::Safe {
            reason: SafeReason::InvalidMarker
        }
    ) {
        return Err(extensions::error_codes::RECOVERY_MARKER_INVALID.to_string());
    }
    extensions::loading_marker::discard_invalid_at(&extensions::loading_marker::path())?;
    state.acknowledge_invalid_marker()?;
    Ok(project(state.inner()))
}

#[tauri::command]
pub fn begin_extension_ui_load(
    state: tauri::State<'_, UiStartupState>,
    acknowledger: tauri::State<'_, UiLoadAcknowledger>,
    extension_id: String,
    attempts: u8,
) -> Result<UiAckToken, String> {
    if !state.loading_allowed_for(&extension_id, attempts) {
        return Err(extensions::error_codes::RECOVERY_MARKER_INVALID.to_string());
    }
    acknowledger.begin(&extension_id, attempts)
}

#[tauri::command]
pub fn advance_extension_ui_load(extension_id: String, stage: String) -> Result<(), String> {
    extensions::loading_marker::ui_advance(&extension_id, &stage)
}

#[tauri::command]
pub fn acknowledge_extension_ui_load(
    state: tauri::State<'_, UiStartupState>,
    acknowledger: tauri::State<'_, UiLoadAcknowledger>,
    extension_id: String,
    token: UiAckToken,
) -> Result<(), String> {
    acknowledger.acknowledge(&extension_id, &token)?;
    state.complete_authorized_load(&extension_id)
}
