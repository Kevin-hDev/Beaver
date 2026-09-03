use std::path::Path;

use super::loading_marker::{self, JournalRead};
use super::ui_startup_state::{SafeReason, UiStartupMode, UiStartupState};

const MAX_STARTUP_ARG_UNITS: usize = 2_048;
pub(crate) const MAX_STARTUP_ARGS: usize = 128;
pub(crate) const SAFE_MODE_SWITCH: &str = "--safe-mode";

#[cfg(test)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ShiftSource {
    MacOs,
    Windows,
    X11,
    Wayland,
}

pub(crate) fn safe_mode_from_args<'a>(
    arguments: impl IntoIterator<Item = &'a str>,
) -> Result<bool, String> {
    let mut found = false;
    for (index, argument) in arguments.into_iter().enumerate() {
        if index >= MAX_STARTUP_ARGS {
            return Err(invalid());
        }
        if index == 0 {
            continue;
        }
        if argument.chars().count() > MAX_STARTUP_ARG_UNITS {
            return Err(invalid());
        }
        if argument == SAFE_MODE_SWITCH {
            if found {
                return Err(invalid());
            }
            found = true;
        } else if argument.starts_with(SAFE_MODE_SWITCH) {
            return Err(invalid());
        }
    }
    Ok(found)
}

pub(crate) fn decide_at(
    marker_path: &Path,
    safe_argument: bool,
    shift: bool,
) -> Result<UiStartupState, String> {
    let mode = if safe_argument {
        UiStartupMode::Safe {
            reason: SafeReason::Argument,
        }
    } else if shift {
        UiStartupMode::Safe {
            reason: SafeReason::Shift,
        }
    } else {
        mode_from_journal(loading_marker::read_journal_at(marker_path))
    };
    Ok(UiStartupState::resolved(mode))
}

pub(crate) fn prepare() -> Result<UiStartupState, String> {
    let arguments = collect_startup_args(std::env::args_os())?;
    prepare_from_args_at(
        &loading_marker::path(),
        arguments.iter().map(String::as_str),
        super::ui_startup_platform::shift_pressed().unwrap_or(false),
        cfg!(target_os = "linux") && std::env::var_os("WAYLAND_DISPLAY").is_some(),
    )
}

pub(crate) fn collect_startup_args(
    arguments: impl IntoIterator<Item = std::ffi::OsString>,
) -> Result<Vec<String>, String> {
    let mut collected = Vec::with_capacity(MAX_STARTUP_ARGS);
    for argument in arguments {
        if collected.len() == MAX_STARTUP_ARGS {
            return Err(invalid());
        }
        collected.push(argument.into_string().map_err(|_| invalid())?);
    }
    Ok(collected)
}

#[cfg(any(target_os = "windows", test))]
pub(crate) fn cef_safe_mode_switch_name() -> &'static str {
    SAFE_MODE_SWITCH
        .strip_prefix("--")
        .unwrap_or(SAFE_MODE_SWITCH)
}

#[cfg(any(target_os = "windows", test))]
pub(crate) fn cef_child_safe_mode_action(present: bool, value: &str) -> Result<bool, String> {
    if !present {
        return Ok(false);
    }
    value.is_empty().then_some(true).ok_or_else(invalid)
}

pub(super) fn prepare_from_args_at<'a>(
    marker_path: &Path,
    arguments: impl IntoIterator<Item = &'a str>,
    shift: bool,
    wayland: bool,
) -> Result<UiStartupState, String> {
    let safe_argument = safe_mode_from_args(arguments)?;
    if safe_argument {
        return Ok(UiStartupState::resolved(UiStartupMode::Safe {
            reason: SafeReason::Argument,
        }));
    }
    if wayland {
        let fallback = mode_from_journal(loading_marker::read_journal_at(marker_path));
        return Ok(UiStartupState::awaiting_wayland(fallback));
    }
    decide_at(marker_path, false, shift)
}

fn mode_from_journal(journal: JournalRead) -> UiStartupMode {
    match journal {
        JournalRead::Missing => UiStartupMode::Normal,
        JournalRead::Invalid => UiStartupMode::Safe {
            reason: SafeReason::InvalidMarker,
        },
        JournalRead::Valid(journal) => journal.ui().map_or(UiStartupMode::Normal, |ui| {
            UiStartupMode::PendingInterruptedUi {
                extension_id: ui.extension_id.clone(),
                stage: ui.stage.clone(),
                attempts: ui.attempts,
            }
        }),
    }
}

#[cfg(test)]
pub(crate) fn unresolved_wayland_state() -> UiStartupState {
    UiStartupState::awaiting_wayland(UiStartupMode::Normal)
}

#[cfg(test)]
pub(crate) fn probe_shift(source: ShiftSource, probe: impl FnOnce() -> Option<bool>) -> bool {
    !matches!(source, ShiftSource::Wayland) && probe().unwrap_or(false)
}

fn invalid() -> String {
    super::error_codes::RECOVERY_MARKER_INVALID.to_string()
}
