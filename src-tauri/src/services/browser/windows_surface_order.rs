use super::surface_bounds::NativeSurfaceRect;
use windows_sys::Win32::Foundation::HWND;
use windows_sys::Win32::UI::WindowsAndMessaging::{
    SetWindowPos, HWND_TOP, SWP_HIDEWINDOW, SWP_NOACTIVATE, SWP_NOOWNERZORDER, SWP_NOZORDER,
    SWP_SHOWWINDOW,
};

pub(super) fn apply_surface_placement(
    window: HWND,
    rect: NativeSurfaceRect,
    visible: bool,
) -> Result<(), ()> {
    let flags = if visible {
        // CEF doit repasser devant le WebView enfant que Wry remonte lors de sa création.
        SWP_NOACTIVATE | SWP_NOOWNERZORDER | SWP_SHOWWINDOW
    } else {
        // Masquer une boîte native ne doit pas modifier l'ordre qui sera restauré à la réouverture.
        SWP_NOACTIVATE | SWP_NOOWNERZORDER | SWP_NOZORDER | SWP_HIDEWINDOW
    };
    let result = unsafe {
        SetWindowPos(
            window,
            HWND_TOP,
            rect.x,
            rect.y,
            rect.width,
            rect.height,
            flags,
        )
    };
    if result == 0 {
        return Err(());
    }
    Ok(())
}
