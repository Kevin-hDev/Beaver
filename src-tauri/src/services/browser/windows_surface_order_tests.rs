use super::{surface_bounds::NativeSurfaceRect, windows_surface_order::apply_surface_placement};
use std::ptr::null_mut;
use windows_sys::Win32::Foundation::{HWND, POINT, RECT};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    ChildWindowFromPoint, CreateWindowExW, DestroyWindow, GetWindow, GetWindowRect,
    IsWindowVisible, SetWindowPos, GW_HWNDNEXT, HWND_TOP, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE,
    WS_CHILD, WS_POPUP, WS_VISIBLE,
};

const STATIC_CLASS: [u16; 7] = [
    b'S' as u16,
    b'T' as u16,
    b'A' as u16,
    b'T' as u16,
    b'I' as u16,
    b'C' as u16,
    0,
];

struct WindowStack {
    parent: HWND,
    browser: HWND,
    webview: HWND,
}

impl WindowStack {
    fn new() -> Self {
        let parent = create_window(
            null_mut(),
            WS_POPUP | WS_VISIBLE,
            -20_000,
            -20_000,
            400,
            300,
        );
        let browser = create_window(parent, WS_CHILD | WS_VISIBLE, 0, 0, 300, 200);
        let webview = create_window(parent, WS_CHILD | WS_VISIBLE, 0, 0, 300, 200);
        let raised = unsafe {
            SetWindowPos(
                webview,
                HWND_TOP,
                0,
                0,
                0,
                0,
                SWP_NOACTIVATE | SWP_NOMOVE | SWP_NOSIZE,
            )
        };
        assert_ne!(raised, 0, "la fenêtre Wry de préparation doit être levée");
        let stack = Self {
            parent,
            browser,
            webview,
        };
        assert_eq!(stack.front_child(10, 10), webview);
        stack
    }

    fn front_child(&self, x: i32, y: i32) -> HWND {
        unsafe { ChildWindowFromPoint(self.parent, POINT { x, y }) }
    }

    fn browser_rect(&self) -> NativeSurfaceRect {
        let mut rect = RECT::default();
        let mut parent_rect = RECT::default();
        let read = unsafe { GetWindowRect(self.browser, &mut rect) };
        assert_ne!(read, 0, "la géométrie CEF doit être lisible");
        let parent_read = unsafe { GetWindowRect(self.parent, &mut parent_rect) };
        assert_ne!(parent_read, 0, "la géométrie du parent doit être lisible");
        NativeSurfaceRect {
            x: rect.left - parent_rect.left,
            y: rect.top - parent_rect.top,
            width: rect.right - rect.left,
            height: rect.bottom - rect.top,
        }
    }
}

impl Drop for WindowStack {
    fn drop(&mut self) {
        unsafe {
            DestroyWindow(self.webview);
            DestroyWindow(self.browser);
            DestroyWindow(self.parent);
        }
    }
}

fn create_window(parent: HWND, style: u32, x: i32, y: i32, width: i32, height: i32) -> HWND {
    let window = unsafe {
        CreateWindowExW(
            0,
            STATIC_CLASS.as_ptr(),
            null_mut(),
            style,
            x,
            y,
            width,
            height,
            parent,
            null_mut(),
            null_mut(),
            null_mut(),
        )
    };
    assert!(
        !window.is_null(),
        "la fenêtre Win32 de test doit être créée"
    );
    window
}

#[test]
fn visible_surface_owns_geometry_visibility_and_child_order() {
    let windows = WindowStack::new();
    let expected = NativeSurfaceRect {
        x: 17,
        y: 23,
        width: 180,
        height: 120,
    };

    assert_eq!(
        apply_surface_placement(windows.browser, expected, true),
        Ok(())
    );
    assert_ne!(unsafe { IsWindowVisible(windows.browser) }, 0);
    assert_eq!(windows.browser_rect(), expected);
    assert_eq!(windows.front_child(20, 25), windows.browser);
}

#[test]
fn hidden_surface_keeps_its_order_and_reopen_restores_the_front() {
    let windows = WindowStack::new();
    let first = NativeSurfaceRect {
        x: 10,
        y: 12,
        width: 190,
        height: 130,
    };
    assert_eq!(
        apply_surface_placement(windows.browser, first, true),
        Ok(())
    );
    assert_eq!(
        unsafe { GetWindow(windows.browser, GW_HWNDNEXT) },
        windows.webview
    );

    let hidden = NativeSurfaceRect {
        x: 20,
        y: 24,
        width: 170,
        height: 110,
    };
    assert_eq!(
        apply_surface_placement(windows.browser, hidden, false),
        Ok(())
    );
    assert_eq!(unsafe { IsWindowVisible(windows.browser) }, 0);
    assert_eq!(windows.browser_rect(), hidden);
    assert_eq!(
        unsafe { GetWindow(windows.browser, GW_HWNDNEXT) },
        windows.webview
    );

    assert_eq!(
        apply_surface_placement(windows.browser, hidden, true),
        Ok(())
    );
    assert_ne!(unsafe { IsWindowVisible(windows.browser) }, 0);
    assert_eq!(windows.front_child(25, 29), windows.browser);
}
