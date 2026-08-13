mod activity;
mod event_mapping;
mod lifecycle;
mod settings;
mod window;

use crate::models::MascotSettings;
use crate::services::agent_local::types_ollama::StreamEvent;
use activity::ActivityArbiter;
pub use activity::{MascotAnimation, MascotStatePayload};
pub use lifecycle::{cancel_session, MascotSession, MascotSessionOutcome};
use std::sync::Mutex;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter, Manager};

pub const STATE_EVENT: &str = "mascot-state-changed";
pub const SETTINGS_EVENT: &str = "mascot-settings-changed";
pub const APP_FOCUS_EVENT: &str = "mascot-app-focus-changed";

pub struct MascotRuntime {
    activity: Mutex<ActivityArbiter>,
    current_settings: Mutex<MascotSettings>,
    window_guard: Mutex<()>,
    mutation_gate: tokio::sync::Mutex<()>,
}

impl Default for MascotRuntime {
    fn default() -> Self {
        Self {
            activity: Mutex::new(ActivityArbiter::default()),
            current_settings: Mutex::new(MascotSettings::default()),
            window_guard: Mutex::new(()),
            mutation_gate: tokio::sync::Mutex::new(()),
        }
    }
}

pub fn initialize(app: &AppHandle, settings: MascotSettings) {
    let settings = settings.normalized();
    let runtime = app.state::<MascotRuntime>();
    if window::apply(app, &runtime, &settings).is_ok() {
        let _ = settings::store_current(&runtime, settings);
    }
}

pub fn start_activity_cleanup(app: &AppHandle) {
    let handle = app.clone();
    let background = app
        .state::<crate::services::runtime_background::RuntimeBackgroundServices>()
        .inner()
        .clone();
    if background
        .spawn_loop(move |cancel| async move {
            let mut interval = tokio::time::interval(Duration::from_millis(250));
            interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
            loop {
                tokio::select! {
                    _ = cancel.cancelled() => return,
                    _ = interval.tick() => refresh_activity(&handle),
                }
            }
        })
        .is_err()
    {
        ::log::warn!("[mascot] activity cleanup unavailable during shutdown");
    }
}

pub fn observe_stream_event(
    app: &AppHandle,
    session_id: &str,
    generation: Option<u64>,
    event: &StreamEvent,
) {
    let Some((animation, ttl, resume_previous)) = event_mapping::animation_for_event(event) else {
        return;
    };
    update_activity(app, |arbiter| {
        arbiter.update(
            session_id,
            generation,
            animation,
            ttl,
            resume_previous,
            Instant::now(),
        )
    });
}

pub fn current_state(app: &AppHandle) -> Result<MascotStatePayload, String> {
    app.state::<MascotRuntime>()
        .activity
        .lock()
        .map(|arbiter| arbiter.state())
        .map_err(|_| generic_error())
}

pub async fn get_settings() -> Result<MascotSettings, String> {
    settings::get().await
}

pub async fn patch_settings(
    app: &AppHandle,
    patch: crate::models::MascotSettingsPatch,
) -> Result<MascotSettings, String> {
    settings::patch(app, patch).await
}

pub async fn save_position(app: &AppHandle, x: i32, y: i32) -> Result<(), String> {
    settings::save_position(app, x, y).await
}

pub async fn sync_from_disk(app: AppHandle) {
    let _ = settings::sync_from_disk(&app).await;
}

pub fn handle_window_focus(app: &AppHandle, focused: bool) {
    if focused {
        let _ = app.emit(APP_FOCUS_EVENT, true);
        return;
    }
    let handle = app.clone();
    let background = app
        .state::<crate::services::runtime_background::RuntimeBackgroundServices>()
        .inner()
        .clone();
    if background
        .spawn_task(move |cancel| async move {
            tokio::select! {
                _ = cancel.cancelled() => return,
                _ = tokio::time::sleep(Duration::from_millis(50)) => {}
            }
            let any_focused = ["main", "mascot"].iter().any(|label| {
                handle
                    .get_webview_window(label)
                    .and_then(|window| window.is_focused().ok())
                    .unwrap_or(false)
            });
            let _ = handle.emit(APP_FOCUS_EVENT, any_focused);
        })
        .is_err()
    {
        ::log::warn!("[mascot] focus refresh unavailable during shutdown");
    }
}

fn refresh_activity(app: &AppHandle) {
    update_activity(app, |arbiter| arbiter.refresh(Instant::now()));
}

pub(super) fn update_activity(
    app: &AppHandle,
    update: impl FnOnce(&mut ActivityArbiter) -> Option<MascotStatePayload>,
) {
    let payload = app
        .state::<MascotRuntime>()
        .activity
        .lock()
        .ok()
        .and_then(|mut arbiter| update(&mut arbiter));
    if let Some(payload) = payload {
        let _ = app.emit(STATE_EVENT, payload);
    }
}

fn generic_error() -> String {
    "Mascotte indisponible".to_string()
}

#[cfg(test)]
mod focus_ownership_tests {
    #[test]
    fn delayed_focus_check_uses_the_runtime_background_owner() {
        let source = include_str!("mod.rs");

        assert_eq!(source.matches("tauri::async_runtime::spawn").count(), 1);
        assert!(source.contains("RuntimeBackgroundServices"));
    }
}
