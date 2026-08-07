use crate::app_events::AUTOSTART_ARG;
use auto_launch::{AutoLaunch, AutoLaunchBuilder};
use std::path::PathBuf;
#[cfg(target_os = "linux")]
use tauri::Manager;
use tauri::Runtime;
use tauri_plugin_autostart::{AutoLaunchManager, ManagerExt};

pub const ACTIVE_ENTRY_NAME: &str = crate::services::brand::DISPLAY_NAME;
const LEGACY_ENTRY_NAME: &str = "CL-GO";
const MARKER_DIRECTORY: &str = "migrations";
const MARKER_FILE: &str = "autostart-beaver-v1";
const CONFIGURATION_ERROR: &str = "configuration_error";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum MigrationError {
    State,
    Setup,
    Marker,
}

trait LaunchEntry {
    fn is_enabled(&self) -> Result<bool, MigrationError>;
    fn enable(&self) -> Result<(), MigrationError>;
    fn disable(&self) -> Result<(), MigrationError>;
}

impl LaunchEntry for AutoLaunchManager {
    fn is_enabled(&self) -> Result<bool, MigrationError> {
        AutoLaunchManager::is_enabled(self).map_err(|_| MigrationError::State)
    }

    fn enable(&self) -> Result<(), MigrationError> {
        AutoLaunchManager::enable(self).map_err(|_| MigrationError::State)
    }

    fn disable(&self) -> Result<(), MigrationError> {
        AutoLaunchManager::disable(self).map_err(|_| MigrationError::State)
    }
}

impl LaunchEntry for AutoLaunch {
    fn is_enabled(&self) -> Result<bool, MigrationError> {
        AutoLaunch::is_enabled(self).map_err(|_| MigrationError::State)
    }

    fn enable(&self) -> Result<(), MigrationError> {
        AutoLaunch::enable(self).map_err(|_| MigrationError::State)
    }

    fn disable(&self) -> Result<(), MigrationError> {
        AutoLaunch::disable(self).map_err(|_| MigrationError::State)
    }
}

pub fn plugin<R: Runtime>() -> tauri::plugin::TauriPlugin<R> {
    let builder = tauri_plugin_autostart::Builder::new()
        .app_name(ACTIVE_ENTRY_NAME)
        .arg(AUTOSTART_ARG);
    #[cfg(target_os = "macos")]
    let builder = builder.macos_launcher(tauri_plugin_autostart::MacosLauncher::LaunchAgent);
    builder.build()
}

pub fn synchronize_at_startup(app: &tauri::AppHandle, requested: bool) {
    if synchronize(app, requested).is_err() {
        ::log::error!("[autostart] synchronization failed");
    }
}

pub fn synchronize_for_settings(app: &tauri::AppHandle, requested: bool) -> Result<(), String> {
    synchronize(app, requested).map_err(|_| CONFIGURATION_ERROR.to_string())
}

fn synchronize(app: &tauri::AppHandle, requested: bool) -> Result<(), MigrationError> {
    let active = app.autolaunch();
    let marker = marker_path();
    if marker.try_exists().map_err(|_| MigrationError::Marker)? {
        return synchronize_entry(active.inner(), requested);
    }

    let legacy = build_legacy_entry(app)?;
    migrate_and_mark(active.inner(), &legacy, requested, || {
        crate::services::private_store::atomic_write(&marker, b"ok")
            .map_err(|_| MigrationError::Marker)
    })
}

fn build_legacy_entry(app: &tauri::AppHandle) -> Result<AutoLaunch, MigrationError> {
    let executable = executable_path(app)?;
    let mut builder = AutoLaunchBuilder::new();
    builder
        .set_app_name(LEGACY_ENTRY_NAME)
        .set_app_path(&executable)
        .set_args(&[AUTOSTART_ARG]);
    #[cfg(target_os = "macos")]
    builder.set_use_launch_agent(true);
    builder.build().map_err(|_| MigrationError::Setup)
}

fn executable_path(app: &tauri::AppHandle) -> Result<String, MigrationError> {
    let current = std::env::current_exe().map_err(|_| MigrationError::Setup)?;
    #[cfg(target_os = "linux")]
    let current = app.env().appimage.map(PathBuf::from).unwrap_or(current);
    #[cfg(not(target_os = "linux"))]
    let _ = app;
    #[cfg(target_os = "macos")]
    let current = current.canonicalize().map_err(|_| MigrationError::Setup)?;
    Ok(current.display().to_string())
}

fn marker_path() -> PathBuf {
    crate::services::paths::data_dir()
        .join(MARKER_DIRECTORY)
        .join(MARKER_FILE)
}

fn migrate_and_mark<A, L, F>(
    active: &A,
    legacy: &L,
    requested: bool,
    write_marker: F,
) -> Result<(), MigrationError>
where
    A: LaunchEntry,
    L: LaunchEntry,
    F: FnOnce() -> Result<(), MigrationError>,
{
    let active_was_enabled = active.is_enabled()?;
    let legacy_was_enabled = legacy.is_enabled()?;

    if requested {
        if ensure_enabled(active, active_was_enabled).is_err() {
            if legacy_was_enabled {
                let _ = ensure_enabled(legacy, false);
            }
            return Err(MigrationError::State);
        }
        if ensure_disabled(legacy).is_err() {
            let _ = ensure_disabled(active);
            return Err(MigrationError::State);
        }
    } else {
        ensure_disabled(active)?;
        ensure_disabled(legacy)?;
    }

    verify_final_state(active, legacy, requested)?;
    write_marker()
}

fn synchronize_entry(entry: &impl LaunchEntry, requested: bool) -> Result<(), MigrationError> {
    let current = entry.is_enabled()?;
    if requested {
        ensure_enabled(entry, current)
    } else {
        ensure_disabled(entry)
    }
}

fn ensure_enabled(entry: &impl LaunchEntry, already_enabled: bool) -> Result<(), MigrationError> {
    if !already_enabled {
        let _ = entry.enable();
    }
    entry
        .is_enabled()?
        .then_some(())
        .ok_or(MigrationError::State)
}

fn ensure_disabled(entry: &impl LaunchEntry) -> Result<(), MigrationError> {
    let _ = entry.disable();
    (!entry.is_enabled()?)
        .then_some(())
        .ok_or(MigrationError::State)
}

fn verify_final_state(
    active: &impl LaunchEntry,
    legacy: &impl LaunchEntry,
    requested: bool,
) -> Result<(), MigrationError> {
    (active.is_enabled()? == requested && !legacy.is_enabled()?)
        .then_some(())
        .ok_or(MigrationError::State)
}

#[cfg(test)]
#[path = "autostart_migration_tests.rs"]
mod tests;
