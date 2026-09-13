use super::{public_error, UpdateProgressRuntime};
use tauri::{
    AppHandle, LogicalSize, Manager, Monitor, PhysicalPosition, PhysicalSize, WebviewUrl,
    WebviewWindow, WebviewWindowBuilder, WindowEvent,
};

pub const WINDOW_LABEL: &str = "update-progress";
const WINDOW_ENTRY: &str = "update-window.html";
pub const WIDTH: f64 = 380.0;
pub const MIN_HEIGHT: u16 = 96;
pub const MAX_HEIGHT: u16 = 640;
const MIN_VISIBLE_PIXELS: i64 = 24;

#[cfg(target_os = "linux")]
pub fn show(_app: &AppHandle, _runtime: &UpdateProgressRuntime) -> Result<(), String> {
    Ok(())
}

#[cfg(not(target_os = "linux"))]
pub fn show(app: &AppHandle, runtime: &UpdateProgressRuntime) -> Result<(), String> {
    if let Some(window) = app.get_webview_window(WINDOW_LABEL) {
        window.show().map_err(|_| public_error())?;
        return window.set_focus().map_err(|_| public_error());
    }
    let window = WebviewWindowBuilder::new(app, WINDOW_LABEL, WebviewUrl::App(WINDOW_ENTRY.into()))
        .title("Beaver")
        .inner_size(WIDTH, f64::from(MIN_HEIGHT))
        .decorations(false)
        .transparent(true)
        .resizable(false)
        .maximizable(false)
        .minimizable(false)
        .always_on_top(true)
        .skip_taskbar(true)
        .visible(false)
        .build()
        .map_err(|_| public_error())?;
    position(&window, runtime.saved_position())?;
    let state = runtime.clone();
    let positioned_window = window.clone();
    window.on_window_event(move |event| {
        if let WindowEvent::Moved(position) = event {
            state.save_position(*position);
            return;
        }
        if matches!(
            event,
            WindowEvent::CloseRequested { .. } | WindowEvent::Destroyed
        ) {
            if let Ok(position) = positioned_window.outer_position() {
                state.save_position(position);
            }
        }
    });
    window.show().map_err(|_| public_error())
}

fn position(window: &WebviewWindow, saved: Option<PhysicalPosition<i32>>) -> Result<(), String> {
    let size = window.inner_size().map_err(|_| public_error())?;
    let monitors = window.available_monitors().map_err(|_| public_error())?;
    let position = saved
        .filter(|position| is_visible(*position, size, &monitors))
        .or_else(|| centered(window, size));
    if let Some(position) = position {
        window.set_position(position).map_err(|_| public_error())?;
    }
    Ok(())
}

fn centered(window: &WebviewWindow, size: PhysicalSize<u32>) -> Option<PhysicalPosition<i32>> {
    let monitor = window.primary_monitor().ok().flatten()?;
    let area = monitor.work_area();
    let x = i64::from(area.position.x) + (i64::from(area.size.width) - i64::from(size.width)) / 2;
    let y = i64::from(area.position.y) + (i64::from(area.size.height) - i64::from(size.height)) / 2;
    Some(PhysicalPosition::new(clamp(x), clamp(y)))
}

fn is_visible(
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

pub fn resize(window: &WebviewWindow, height: u16) -> Result<(), String> {
    if window.label() != WINDOW_LABEL || !(MIN_HEIGHT..=MAX_HEIGHT).contains(&height) {
        return Err("command-not-available".to_string());
    }
    let monitor = window.current_monitor().map_err(|_| public_error())?;
    let max_height = monitor
        .map(|monitor| {
            (f64::from(monitor.work_area().size.height) / monitor.scale_factor()).floor() as u16
        })
        .unwrap_or(MAX_HEIGHT)
        .min(MAX_HEIGHT);
    window
        .set_size(LogicalSize::new(WIDTH, f64::from(height.min(max_height))))
        .map_err(|_| public_error())
}

fn clamp(value: i64) -> i32 {
    value.clamp(i64::from(i32::MIN), i64::from(i32::MAX)) as i32
}
