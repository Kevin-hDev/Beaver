use super::cef_cookie_gate::{CookieGateContext, PROBE_COOKIE_NAME, PROBE_COOKIE_URL};
use super::cef_cookie_gate_policy::cef_cookie_gate_policy;
#[cfg(target_os = "macos")]
use super::cookie_store_probe::cookie_store_hides_probe;
use cef::*;

pub(super) fn flush_and_check(context: CookieGateContext) {
    let Some(manager) = cookie_manager_get_global_manager(None) else {
        context.complete(false);
        return;
    };
    let mut callback = ProbeStoreFlushed::new(context.clone());
    if manager.flush_store(Some(&mut callback)) != 1 {
        delete_probe(context, false);
    }
}

wrap_completion_callback! {
    struct ProbeStoreFlushed {
        context: CookieGateContext,
    }

    impl CompletionCallback {
        fn on_complete(&self) {
            let failed = self.context.clone();
            super::ffi_guard::unit_or(
                || failed.complete(false),
                || {
                    let secure = verify_flushed_store(&self.context);
                    delete_probe(self.context.clone(), secure);
                },
            );
        }
    }
}

fn verify_flushed_store(_context: &CookieGateContext) -> bool {
    if !cef_cookie_gate_policy().verify_store_on_disk {
        // Chromium 132+ locks the live Cookies database exclusively on Windows.
        // Successful CEF set/flush/delete callbacks are the supported live check.
        return true;
    }

    #[cfg(target_os = "macos")]
    {
        _context
            .probe_copy()
            .and_then(|probe| cookie_store_hides_probe(_context.profile(), probe.as_ref()))
            .unwrap_or(false)
    }

    #[cfg(target_os = "windows")]
    false
}

fn delete_probe(context: CookieGateContext, secure: bool) {
    let Some(manager) = cookie_manager_get_global_manager(None) else {
        context.complete(false);
        return;
    };
    let url = CefString::from(PROBE_COOKIE_URL);
    let name = CefString::from(PROBE_COOKIE_NAME);
    let mut callback = ProbeDeleted::new(context.clone(), secure);
    if manager.delete_cookies(Some(&url), Some(&name), Some(&mut callback)) != 1 {
        context.complete(false);
    }
}

wrap_delete_cookies_callback! {
    struct ProbeDeleted {
        context: CookieGateContext,
        secure: bool,
    }

    impl DeleteCookiesCallback {
        fn on_complete(&self, num_deleted: i32) {
            let failed = self.context.clone();
            super::ffi_guard::unit_or(
                || failed.complete(false),
                || {
                    if num_deleted < 1 {
                        self.context.complete(false);
                        return;
                    }
                    flush_cleanup(self.context.clone(), self.secure);
                },
            );
        }
    }
}

fn flush_cleanup(context: CookieGateContext, secure: bool) {
    let Some(manager) = cookie_manager_get_global_manager(None) else {
        context.complete(false);
        return;
    };
    let mut callback = CleanupFlushed::new(context.clone(), secure);
    if manager.flush_store(Some(&mut callback)) != 1 {
        context.complete(false);
    }
}

wrap_completion_callback! {
    struct CleanupFlushed {
        context: CookieGateContext,
        secure: bool,
    }

    impl CompletionCallback {
        fn on_complete(&self) {
            let failed = self.context.clone();
            super::ffi_guard::unit_or(
                || failed.complete(false),
                || self.context.complete(self.secure),
            );
        }
    }
}
