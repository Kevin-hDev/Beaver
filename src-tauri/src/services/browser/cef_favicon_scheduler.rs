use super::{cef_engine, cef_favicon_handler, favicon_runtime::FAVICONS};
use cef::*;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Instant;

static SCHEDULED: AtomicBool = AtomicBool::new(false);

cef::wrap_task! {
    struct FaviconPump { app: tauri::AppHandle }
    impl Task {
        fn execute(&self) {
            super::ffi_guard::unit(|| {
                SCHEDULED.store(false, Ordering::Release);
                let jobs = match FAVICONS.lock() {
                    Ok(mut state) => state.take_ready(Instant::now()),
                    Err(_) => return,
                };
                for job in jobs {
                    let browser = cef_engine::favicon_browser(&job.key, job.ticket.view_epoch);
                    cef_favicon_handler::download(&self.app, job, browser);
                }
            });
        }
    }
}

pub(super) fn schedule(app: &tauri::AppHandle) {
    if SCHEDULED.swap(true, Ordering::AcqRel) {
        return;
    }
    let mut task = FaviconPump::new(app.clone());
    // Post, never inline: CEF can notify while the surface registry is borrowed.
    if post_task(ThreadId::UI, Some(&mut task)) != 1 {
        SCHEDULED.store(false, Ordering::Release);
        log::debug!("[browser] favicon scheduling unavailable");
    }
}
