use super::cef_cookie_gate_cleanup;
use super::runtime_handle::BrowserRuntimeHandle;
use cef::*;
use rand::{rngs::OsRng, RngCore};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tauri::Emitter;
use zeroize::{Zeroize, Zeroizing};

pub(super) const PROBE_COOKIE_URL: &str = "https://clgo.invalid/";
pub(super) const PROBE_COOKIE_NAME: &str = "__clgo_cef_encryption_probe";
const GATE_TIMEOUT: Duration = Duration::from_secs(60);

#[derive(Clone)]
pub(super) struct CookieGateContext {
    app: tauri::AppHandle,
    #[cfg(target_os = "macos")]
    profile: PathBuf,
    runtime: BrowserRuntimeHandle,
    secret: Arc<Mutex<Zeroizing<String>>>,
    completed: Arc<AtomicBool>,
}

impl CookieGateContext {
    fn new(app: tauri::AppHandle, profile: PathBuf, runtime: BrowserRuntimeHandle) -> Self {
        #[cfg(target_os = "windows")]
        drop(profile);
        Self {
            app,
            #[cfg(target_os = "macos")]
            profile,
            runtime,
            secret: Arc::new(Mutex::new(generate_probe_value())),
            completed: Arc::new(AtomicBool::new(false)),
        }
    }

    #[cfg(target_os = "macos")]
    pub(super) fn profile(&self) -> &PathBuf {
        &self.profile
    }

    #[cfg(target_os = "macos")]
    pub(super) fn probe_copy(&self) -> Result<Zeroizing<Vec<u8>>, ()> {
        let secret = self.secret.lock().map_err(|_| ())?;
        Ok(Zeroizing::new(secret.as_bytes().to_vec()))
    }

    pub(super) fn complete(&self, secure: bool) {
        if self
            .completed
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .is_err()
        {
            return;
        }
        if let Ok(mut secret) = self.secret.lock() {
            secret.zeroize();
        }
        if secure {
            let _ = self.runtime.mark_running();
        } else {
            let _ = self.runtime.mark_failed();
        }
        let _ = self.app.emit(
            "browser-capability-v1",
            super::capability_for_runtime(&self.runtime),
        );
    }
}

pub(super) fn start(app: tauri::AppHandle, profile: PathBuf, runtime: BrowserRuntimeHandle) {
    let context = CookieGateContext::new(app, profile, runtime);
    let timeout_context = context.clone();
    std::thread::spawn(move || {
        std::thread::sleep(GATE_TIMEOUT);
        timeout_context.complete(false);
    });
    let mut callback = CookieManagerReady::new(context.clone());
    if cookie_manager_get_global_manager(Some(&mut callback)).is_none() {
        context.complete(false);
    }
}

fn generate_probe_value() -> Zeroizing<String> {
    let mut random = Zeroizing::new([0_u8; 32]);
    OsRng.fill_bytes(random.as_mut());
    Zeroizing::new(hex::encode(random.as_ref()))
}

wrap_completion_callback! {
    struct CookieManagerReady {
        context: CookieGateContext,
    }

    impl CompletionCallback {
        fn on_complete(&self) {
            let failed = self.context.clone();
            super::ffi_guard::unit_or(
                || failed.complete(false),
                || set_probe_cookie(self.context.clone()),
            );
        }
    }
}

fn set_probe_cookie(context: CookieGateContext) {
    let Some(manager) = cookie_manager_get_global_manager(None) else {
        context.complete(false);
        return;
    };
    let Ok(secret) = context.secret.lock() else {
        context.complete(false);
        return;
    };
    let mut cookie = Cookie {
        name: CefString::from(PROBE_COOKIE_NAME),
        value: CefString::from(secret.as_str()),
        path: CefString::from("/"),
        secure: 1,
        httponly: 1,
        same_site: CookieSameSite::LAX_MODE,
        ..Default::default()
    };
    drop(secret);
    let url = CefString::from(PROBE_COOKIE_URL);
    let mut callback = ProbeCookieSet::new(context.clone());
    let accepted = manager.set_cookie(Some(&url), Some(&cookie), Some(&mut callback)) == 1;
    clear_sensitive_cef_string(&mut cookie.value);
    if !accepted {
        context.complete(false);
    }
}

wrap_set_cookie_callback! {
    struct ProbeCookieSet {
        context: CookieGateContext,
    }

    impl SetCookieCallback {
        fn on_complete(&self, success: i32) {
            let failed = self.context.clone();
            super::ffi_guard::unit_or(
                || failed.complete(false),
                || {
                    if success == 1 {
                        cef_cookie_gate_cleanup::flush_and_check(self.context.clone());
                    } else {
                        self.context.complete(false);
                    }
                },
            );
        }
    }
}

fn clear_sensitive_cef_string(value: &mut CefString) {
    let raw: *mut cef::sys::_cef_string_utf16_t = value.into();
    let Some(raw) = (unsafe { raw.as_mut() }) else {
        return;
    };
    if !raw.str_.is_null() && raw.length > 0 {
        let utf16 = unsafe { std::slice::from_raw_parts_mut(raw.str_, raw.length) };
        utf16.zeroize();
    }
    unsafe { cef::sys::cef_string_utf16_clear(raw) };
}
