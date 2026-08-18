use objc2::runtime::{AnyObject, Imp, Sel};
use objc2::{ffi, msg_send, MainThreadMarker};
use objc2_app_kit::NSApp;
use std::sync::OnceLock;

static APP_HANDLE: OnceLock<tauri::AppHandle<tauri::Wry>> = OnceLock::new();

pub(super) fn install(app_handle: &tauri::AppHandle) -> Result<(), ()> {
    APP_HANDLE.set(app_handle.clone()).map_err(|_| ())?;
    let marker = MainThreadMarker::new().ok_or(())?;
    let application = NSApp(marker);
    let delegate: *mut AnyObject = unsafe { msg_send![&*application, delegate] };
    let delegate = unsafe { delegate.as_ref() }.ok_or(())?;
    let class = delegate.class() as *const _ as *mut _;
    let implementation: Imp = unsafe {
        std::mem::transmute::<
            unsafe extern "C-unwind" fn(*mut AnyObject, Sel, *mut AnyObject) -> usize,
            Imp,
        >(application_should_terminate)
    };

    // Cocoa sends external Quit requests directly to the delegate. Returning
    // Cancel keeps the event loop alive while the shared exit authority works.
    unsafe {
        ffi::class_replaceMethod(
            class,
            objc2::sel!(applicationShouldTerminate:),
            implementation,
            c"Q@:@".as_ptr(),
        );
    }
    Ok(())
}

unsafe extern "C-unwind" fn application_should_terminate(
    _delegate: *mut AnyObject,
    _selector: Sel,
    _application: *mut AnyObject,
) -> usize {
    let result = std::panic::catch_unwind(|| {
        let app_handle = APP_HANDLE.get().ok_or(())?;
        ::log::info!("[exit] native termination requested");
        crate::app_exit::request(app_handle, 0);
        Ok::<(), ()>(())
    });
    if !matches!(result, Ok(Ok(()))) {
        eprintln!("[exit] native termination coordination unavailable");
    }
    0 // NSTerminateCancel: only app_exit may end the process.
}

#[cfg(test)]
mod tests {
    #[test]
    fn external_termination_is_cancelled_and_routed_to_the_exit_authority() {
        let source = include_str!("macos_termination.rs");
        let production = source
            .split("#[cfg(test)]")
            .next()
            .expect("production source");
        let callback = production
            .split("unsafe extern \"C-unwind\" fn application_should_terminate")
            .nth(1)
            .expect("termination callback");

        assert!(production.contains("sel!(applicationShouldTerminate:)"));
        assert!(callback.contains("crate::app_exit::request(app_handle, 0)"));
        assert!(callback.contains("0 // NSTerminateCancel"));
    }
}
