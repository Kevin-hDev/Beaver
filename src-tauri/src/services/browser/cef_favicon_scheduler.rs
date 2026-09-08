use super::{
    cef_engine, cef_favicon_handler, favicon_runtime::access, favicon_task_gate::TaskReservation,
};
use cef::*;
use std::{
    sync::{atomic::AtomicBool, Arc},
    time::Instant,
};

static SCHEDULED: AtomicBool = AtomicBool::new(false);

cef::wrap_task! {
    struct FaviconPump { app: tauri::AppHandle, reservation: Arc<TaskReservation<'static>> }
    impl Task {
        fn execute(&self) {
            super::ffi_guard::unit(|| {
                self.reservation.release();
                let jobs = access(Some(&self.app), |state| state.take_available(Instant::now(), |key, epoch| {
                    cef_engine::favicon_browser(key, epoch).and_then(|browser| browser.host()).is_some()
                }));
                for job in jobs {
                    let browser = cef_engine::favicon_browser(&job.key, job.ticket.view_epoch);
                    cef_favicon_handler::download(&self.app, job, browser);
                }
            });
        }
    }
}

pub(super) fn schedule(app: &tauri::AppHandle) {
    let Some(reservation) = TaskReservation::acquire(&SCHEDULED) else {
        return;
    };
    let mut task = FaviconPump::new(app.clone(), Arc::new(reservation));
    // Post, never inline: CEF can notify while the surface registry is borrowed.
    if post_task(ThreadId::UI, Some(&mut task)) != 1 {
        log::debug!("[browser] favicon scheduling unavailable");
    }
}
