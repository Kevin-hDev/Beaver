use super::{generic_error, MascotRuntime};
use crate::models::{MascotPosition, MascotSettings};
use crate::services::brand::DISPLAY_NAME;
use tauri::{
    utils::config::BackgroundThrottlingPolicy, AppHandle, CursorIcon, LogicalSize, Manager,
    Monitor, PhysicalPosition, PhysicalSize, WebviewUrl, WebviewWindow, WebviewWindowBuilder,
};

const WINDOW_LABEL: &str = "mascot";
const WINDOW_ENTRY: &str = "mascot.html";
const BASE_WIDTH: f64 = 112.0;
const FRAME_RATIO: f64 = 208.0 / 192.0;
const SCREEN_MARGIN: f64 = 28.0;
const MIN_VISIBLE_PIXELS: i64 = 24;

pub fn apply(
    app: &AppHandle,
    runtime: &MascotRuntime,
    settings: &MascotSettings,
) -> Result<(), String> {
    let _window_operation = runtime.window_guard.lock().map_err(|_| generic_error())?;
    if !settings.enabled {
        if let Some(window) = app.get_webview_window(WINDOW_LABEL) {
            window.close().map_err(|_| generic_error())?;
        }
        return Ok(());
    }

    if let Some(window) = app.get_webview_window(WINDOW_LABEL) {
        configure_existing(&window, settings)?;
        return window.show().map_err(|_| generic_error());
    }

    create(app, settings)
}

fn create(app: &AppHandle, settings: &MascotSettings) -> Result<(), String> {
    let (width, height) = logical_dimensions(settings.size_percent);
    let window = WebviewWindowBuilder::new(app, WINDOW_LABEL, WebviewUrl::App(WINDOW_ENTRY.into()))
        .title(DISPLAY_NAME)
        .inner_size(width, height)
        .decorations(false)
        .transparent(true)
        .shadow(false)
        .resizable(false)
        .maximizable(false)
        .minimizable(false)
        .closable(false)
        .focused(false)
        .focusable(false)
        .accept_first_mouse(true)
        .background_throttling(BackgroundThrottlingPolicy::Disabled)
        .always_on_top(true)
        .visible_on_all_workspaces(true)
        .skip_taskbar(true)
        .visible(false)
        .build()
        .map_err(|_| generic_error())?;

    window
        .set_cursor_icon(CursorIcon::Grab)
        .map_err(|_| generic_error())?;
    position_new_window(&window, settings.position)?;
    window.show().map_err(|_| generic_error())
}

fn configure_existing(window: &WebviewWindow, settings: &MascotSettings) -> Result<(), String> {
    let (width, height) = logical_dimensions(settings.size_percent);
    window
        .set_size(LogicalSize::new(width, height))
        .map_err(|_| generic_error())?;
    window
        .set_always_on_top(true)
        .map_err(|_| generic_error())?;
    window.set_focusable(false).map_err(|_| generic_error())?;
    window
        .set_cursor_icon(CursorIcon::Grab)
        .map_err(|_| generic_error())?;
    let _ = window.set_visible_on_all_workspaces(true);
    let _ = window.set_skip_taskbar(true);
    Ok(())
}

fn logical_dimensions(size_percent: u16) -> (f64, f64) {
    let width = BASE_WIDTH * f64::from(size_percent) / 100.0;
    (width, width * FRAME_RATIO)
}

fn position_new_window(
    window: &WebviewWindow,
    saved: Option<MascotPosition>,
) -> Result<(), String> {
    let size = window.inner_size().map_err(|_| generic_error())?;
    let monitors = window.available_monitors().map_err(|_| generic_error())?;
    let saved = saved.map(|position| PhysicalPosition::new(position.x, position.y));
    let position = saved
        .filter(|position| position_is_visible(*position, size, &monitors))
        .or_else(|| default_position(window, size));
    if let Some(position) = position {
        window.set_position(position).map_err(|_| generic_error())?;
    }
    Ok(())
}

fn default_position(
    window: &WebviewWindow,
    window_size: PhysicalSize<u32>,
) -> Option<PhysicalPosition<i32>> {
    let monitor = window.primary_monitor().ok().flatten()?;
    let area = monitor.work_area();
    let margin = (SCREEN_MARGIN * monitor.scale_factor()).round() as i64;
    let x = i64::from(area.position.x) + i64::from(area.size.width)
        - i64::from(window_size.width)
        - margin;
    let y = i64::from(area.position.y) + i64::from(area.size.height)
        - i64::from(window_size.height)
        - margin;
    Some(PhysicalPosition::new(clamp_i64(x), clamp_i64(y)))
}

fn position_is_visible(
    position: PhysicalPosition<i32>,
    size: PhysicalSize<u32>,
    monitors: &[Monitor],
) -> bool {
    monitors.iter().any(|monitor| {
        let area = monitor.work_area();
        let left = i64::from(position.x).max(i64::from(area.position.x));
        let top = i64::from(position.y).max(i64::from(area.position.y));
        let right = (i64::from(position.x) + i64::from(size.width))
            .min(i64::from(area.position.x) + i64::from(area.size.width));
        let bottom = (i64::from(position.y) + i64::from(size.height))
            .min(i64::from(area.position.y) + i64::from(area.size.height));
        right - left >= MIN_VISIBLE_PIXELS && bottom - top >= MIN_VISIBLE_PIXELS
    })
}

fn clamp_i64(value: i64) -> i32 {
    value.clamp(i64::from(i32::MIN), i64::from(i32::MAX)) as i32
}
