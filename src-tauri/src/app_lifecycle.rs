#[cfg(target_os = "macos")]
use crate::services::browser::BrowserLibraryGuard;

pub(crate) fn run(
    app: tauri::App,
    #[cfg(target_os = "macos")] browser_library: Option<BrowserLibraryGuard>,
) -> i32 {
    let app_handle = app.handle().clone();
    #[cfg(target_os = "macos")]
    let browser_library = std::rc::Rc::new(std::cell::RefCell::new(browser_library));
    #[cfg(target_os = "macos")]
    let event_browser_library = std::rc::Rc::clone(&browser_library);
    crate::startup::run_before_browser_shutdown(
        || {
            app.run_return(move |app_handle, event| {
                crate::services::browser::setup_on_run_event(
                    app_handle,
                    &event,
                    #[cfg(target_os = "macos")]
                    event_browser_library.borrow().as_ref(),
                );
                crate::app_events::handle_run_event(app_handle, event);
            })
        },
        || {
            #[cfg(target_os = "macos")]
            {
                let browser_library = browser_library.borrow_mut().take();
                crate::startup::shutdown_before_library_unload(browser_library, || {
                    crate::services::browser::shutdown(&app_handle);
                });
            }
            #[cfg(not(target_os = "macos"))]
            crate::services::browser::shutdown(&app_handle);
        },
        || crate::app_exit::post_event_loop(&app_handle),
    )
}
