use super::super::surface_bounds::{BrowserSurfaceBounds, NativeSurfaceRect};
use cef::{Browser, ImplBrowser, ImplBrowserHost, Rect, RuntimeStyle, WindowInfo};
use objc2::MainThreadMarker;
use objc2_app_kit::NSView;
use objc2_foundation::{NSPoint, NSRect, NSSize};
use tauri::Manager;

pub(crate) struct NativeParent {
    view: *mut std::ffi::c_void,
    rect: NativeSurfaceRect,
}

impl NativeParent {
    pub(crate) fn window_info(&self) -> WindowInfo {
        let mut info = WindowInfo::default().set_as_child(
            self.view,
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
    MainThreadMarker::new().ok_or(())?;
    let host = browser.host().ok_or(())?;
    host.was_hidden(1);
    let pointer = host.window_handle();
    let view = unsafe { pointer.cast::<NSView>().as_ref() }.ok_or(())?;
    view.setHidden(true);
    view.removeFromSuperview();
    Ok(())
}

pub(crate) fn resolve_parent(
    app: &tauri::AppHandle,
    bounds: &BrowserSurfaceBounds,
) -> Result<NativeParent, ()> {
    MainThreadMarker::new().ok_or(())?;
    let pointer = app
        .get_webview_window("main")
        .ok_or(())?
        .ns_view()
        .map_err(|_| ())?;
    let parent = unsafe { pointer.cast::<NSView>().as_ref() }.ok_or(())?;
    let height = parent.bounds().size.height;
    if !height.is_finite() || height <= 0.0 || height > f64::from(i32::MAX) {
        return Err(());
    }
    Ok(NativeParent {
        view: pointer,
        rect: bounds.native_rect(height.round() as i32)?,
    })
}

pub(crate) fn update_browser(
    app: &tauri::AppHandle,
    browser: &Browser,
    bounds: &BrowserSurfaceBounds,
) -> Result<(), ()> {
    let parent = resolve_parent(app, bounds)?;
    let host = browser.host().ok_or(())?;
    let pointer = host.window_handle();
    let view = unsafe { pointer.cast::<NSView>().as_ref() }.ok_or(())?;
    view.setFrame(NSRect::new(
        NSPoint::new(f64::from(parent.rect.x), f64::from(parent.rect.y)),
        NSSize::new(f64::from(parent.rect.width), f64::from(parent.rect.height)),
    ));
    view.setHidden(!bounds.visible);
    host.was_hidden(i32::from(!bounds.visible));
    host.was_resized();
    Ok(())
}
