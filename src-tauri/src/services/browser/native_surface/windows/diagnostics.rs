use super::super::super::surface_bounds::NativeSurfaceRect;
use windows_sys::Win32::Foundation::{HWND, POINT, RECT};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    ChildWindowFromPoint, GetClassNameW, GetClientRect, GetParent, GetWindow, GetWindowRect,
    IsWindowVisible, GW_HWNDNEXT, GW_HWNDPREV,
};

pub(super) fn log_surface_state(parent: HWND, browser: HWND, expected: NativeSurfaceRect) {
    let mut screen_rect = RECT::default();
    let mut client_rect = RECT::default();
    let center = POINT {
        x: expected.x.saturating_add(expected.width / 2),
        y: expected.y.saturating_add(expected.height / 2),
    };
    let (front, above, below, actual_parent, visible, screen_ok, client_ok) = unsafe {
        (
            ChildWindowFromPoint(parent, center),
            GetWindow(browser, GW_HWNDPREV),
            GetWindow(browser, GW_HWNDNEXT),
            GetParent(browser),
            IsWindowVisible(browser) != 0,
            GetWindowRect(browser, &mut screen_rect) != 0,
            GetClientRect(browser, &mut client_rect) != 0,
        )
    };
    log::info!(
        "[browser] Windows surface expected=({},{} {}x{}) screen={} client={} visible={} parent_ok={} front_is_cef={} front={} above={} below={}",
        expected.x,
        expected.y,
        expected.width,
        expected.height,
        format_rect(screen_ok, screen_rect),
        format_rect(client_ok, client_rect),
        visible,
        actual_parent == parent,
        front == browser,
        window_class(front),
        window_class(above),
        window_class(below),
    );
}

fn format_rect(valid: bool, rect: RECT) -> String {
    if !valid {
        return "unavailable".to_string();
    }
    format!(
        "({},{},{}x{})",
        rect.left,
        rect.top,
        rect.right.saturating_sub(rect.left),
        rect.bottom.saturating_sub(rect.top),
    )
}

fn window_class(window: HWND) -> String {
    if window.is_null() {
        return "none".to_string();
    }
    let mut buffer = [0_u16; 128];
    let length = unsafe { GetClassNameW(window, buffer.as_mut_ptr(), buffer.len() as i32) };
    if length <= 0 {
        return "unknown".to_string();
    }
    String::from_utf16_lossy(&buffer[..length as usize])
}
