use super::pump_scheduler::PumpScheduler;
use super::runtime_handle::BrowserRuntimeHandle;
use cef::*;
use std::path::PathBuf;

wrap_app! {
    pub(super) struct BrowserApp {
        pump: PumpScheduler,
        runtime: BrowserRuntimeHandle,
        profile: PathBuf,
    }

    impl App {
        fn browser_process_handler(&self) -> Option<BrowserProcessHandler> {
            super::ffi_guard::value(None, || Some(BrowserProcessCallbacks::new(
                self.pump.clone(),
                self.runtime.clone(),
                self.profile.clone(),
            )))
        }
    }
}

wrap_browser_process_handler! {
    struct BrowserProcessCallbacks {
        pump: PumpScheduler,
        runtime: BrowserRuntimeHandle,
        profile: PathBuf,
    }

    impl BrowserProcessHandler {
        fn on_context_initialized(&self) {
            let runtime = self.runtime.clone();
            super::ffi_guard::unit_or(
                || {
                    let _ = runtime.mark_failed();
                },
                || {
                    super::cef_cookie_gate::start(
                        self.pump.app().clone(),
                        self.profile.clone(),
                        self.runtime.clone(),
                    );
                },
            );
        }

        fn on_schedule_message_pump_work(&self, delay_ms: i64) {
            super::ffi_guard::unit(|| self.pump.schedule(delay_ms));
        }
    }
}
