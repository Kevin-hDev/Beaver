use super::super::surface_bounds::{BrowserSurfaceBounds, NativeSurfaceRect};
use cef::{Browser, ImplBrowser, ImplBrowserHost, Rect, RuntimeStyle, WindowInfo};
use tauri::Manager;
use windows_sys::Win32::UI::WindowsAndMessaging::{
    DestroyWindow, SetWindowPos, ShowWindow, SWP_NOACTIVATE, SWP_NOOWNERZORDER, SWP_NOZORDER,
    SW_HIDE, SW_SHOWNA,
};

pub(crate) struct NativeParent {
    window: cef::sys::cef_window_handle_t,
    rect: NativeSurfaceRect,
}

impl NativeParent {
    pub(crate) fn window_info(&self) -> WindowInfo {
        let mut info = WindowInfo::default().set_as_child(
            self.window,
            &Rect {
                x: self.rect.x,
                y: self.rect.y,
                width: self.rect.width,
                height: self.rect.height,
            },
        );
        info.runtime_style = RuntimeStyle::ALLOY;
        info
    }
}

pub(crate) fn destroy_browser(browser: &Browser) -> Result<(), ()> {
    let host = browser.host().ok_or(())?;
    host.was_hidden(1);
    let handle = host.window_handle();
    if handle.0.is_null() {
        return Err(());
    }
    (unsafe { DestroyWindow(handle.0.cast()) } != 0)
        .then_some(())
        .ok_or(())
}

pub(crate) fn resolve_parent(
    app: &tauri::AppHandle,
    bounds: &BrowserSurfaceBounds,
) -> Result<NativeParent, ()> {
    let window = app.get_webview_window("main").ok_or(())?;
    let scale_factor = window.scale_factor().map_err(|_| ())?;
    let handle = window.hwnd().map_err(|_| ())?.0;
    if handle.is_null() {
        return Err(());
    }
    Ok(NativeParent {
        window: cef::sys::HWND(handle.cast()),
        rect: bounds.scaled_top_left_rect(scale_factor)?,
    })
}

pub(crate) fn update_browser(
    app: &tauri::AppHandle,
    browser: &Browser,
    bounds: &BrowserSurfaceBounds,
) -> Result<(), ()> {
    let parent = resolve_parent(app, bounds)?;
    let host = browser.host().ok_or(())?;
    let handle = host.window_handle();
    if handle.0.is_null() {
        return Err(());
    }
    let native_handle = handle.0.cast();
    let moved = unsafe {
        SetWindowPos(
            native_handle,
            std::ptr::null_mut(),
            parent.rect.x,
            parent.rect.y,
            parent.rect.width,
            parent.rect.height,
            SWP_NOACTIVATE | SWP_NOOWNERZORDER | SWP_NOZORDER,
        )
    };
    if moved == 0 {
        return Err(());
    }
    unsafe {
        ShowWindow(
            native_handle,
            if bounds.visible { SW_SHOWNA } else { SW_HIDE },
        );
    }
    host.was_hidden(i32::from(!bounds.visible));
    host.was_resized();
    Ok(())
}
