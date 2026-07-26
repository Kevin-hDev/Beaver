mod activity;
mod settings;
mod window;

use crate::models::MascotSettings;
use crate::services::agent_local::types_ollama::StreamEvent;
use activity::ActivityArbiter;
pub use activity::{MascotAnimation, MascotStatePayload};
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

pub fn start_activity_cleanup(app: AppHandle) {
    tauri::async_runtime::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_millis(250));
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        loop {
            interval.tick().await;
            refresh_activity(&app);
        }
    });
}

pub fn observe_stream_event(app: &AppHandle, session_id: &str, event: &StreamEvent) {
    let Some((animation, ttl, resume_previous)) = animation_for_event(event) else {
        return;
    };
    update_activity(app, |arbiter| {
        arbiter.update(session_id, animation, ttl, resume_previous, Instant::now())
    });
}

pub fn start_session(app: &AppHandle, session_id: &str) {
    update_activity(app, |arbiter| {
        arbiter.update(
            session_id,
            MascotAnimation::Thinking,
            None,
            false,
            Instant::now(),
        )
    });
}

pub fn end_session(app: &AppHandle, session_id: &str) {
    update_activity(app, |arbiter| arbiter.remove(session_id, Instant::now()));
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
    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(Duration::from_millis(50)).await;
        let any_focused = ["main", "mascot"].iter().any(|label| {
            handle
                .get_webview_window(label)
                .and_then(|window| window.is_focused().ok())
                .unwrap_or(false)
        });
        let _ = handle.emit(APP_FOCUS_EVENT, any_focused);
    });
}

fn refresh_activity(app: &AppHandle) {
    update_activity(app, |arbiter| arbiter.refresh(Instant::now()));
}

fn update_activity(
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

fn animation_for_event(event: &StreamEvent) -> Option<(MascotAnimation, Option<Duration>, bool)> {
    let persistent = |animation| Some((animation, None, false));
    match event {
        StreamEvent::Thinking { .. }
        | StreamEvent::Token { .. }
        | StreamEvent::ContentPhase { .. } => persistent(MascotAnimation::Thinking),
        StreamEvent::ToolCall { name, .. } | StreamEvent::ToolResult { name, .. } => {
            persistent(tool_animation(name))
        }
        StreamEvent::Compressing { .. } | StreamEvent::SubagentSpawned { .. } => {
            persistent(MascotAnimation::WorkLaptop)
        }
        StreamEvent::PermissionRequest { .. } | StreamEvent::InteractiveChoiceRequest { .. } => {
            persistent(MascotAnimation::Waiting)
        }
        StreamEvent::Done { .. } => Some((
            MascotAnimation::Success,
            Some(Duration::from_millis(2200)),
            false,
        )),
        StreamEvent::Error { .. } => Some((
            MascotAnimation::Failed,
            Some(Duration::from_millis(2600)),
            false,
        )),
        StreamEvent::RetryIndicator { .. } | StreamEvent::Notice { .. } => Some((
            MascotAnimation::Alert,
            Some(Duration::from_millis(1800)),
            true,
        )),
        _ => None,
    }
}

fn tool_animation(name: &str) -> MascotAnimation {
    match name {
        "read_file" | "read_document" | "read_spreadsheet" | "read_image" | "list_dir" | "grep"
        | "glob" | "web_search" | "web_fetch" => MascotAnimation::ExploreBook,
        _ => MascotAnimation::WorkLaptop,
    }
}

fn generic_error() -> String {
    "Mascotte indisponible".to_string()
}

#[cfg(test)]
#[path = "mapping_tests.rs"]
mod mapping_tests;
