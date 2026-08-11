use super::cef_child_admission::BrowserCefSupervision;
use super::pump_scheduler::PumpScheduler;
use super::runtime_handle::BrowserRuntimeHandle;
use cef::*;
use std::path::PathBuf;

wrap_app! {
    pub(super) struct BrowserApp {
        pump: PumpScheduler,
        runtime: BrowserRuntimeHandle,
        profile: PathBuf,
        supervision: BrowserCefSupervision,
    }

    impl App {
        fn browser_process_handler(&self) -> Option<BrowserProcessHandler> {
            super::ffi_guard::value(None, || Some(BrowserProcessCallbacks::new(
                self.pump.clone(),
                self.runtime.clone(),
                self.profile.clone(),
                self.supervision.clone(),
            )))
        }
    }
}

wrap_browser_process_handler! {
    struct BrowserProcessCallbacks {
        pump: PumpScheduler,
        runtime: BrowserRuntimeHandle,
        profile: PathBuf,
        supervision: BrowserCefSupervision,
    }

    impl BrowserProcessHandler {
        fn on_context_initialized(&self) {
            let runtime = self.runtime.clone();
            super::ffi_guard::unit_or(
                || {
                    ::log::error!("[browser] launch callback failed");
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

        #[cfg(any(target_os = "windows", target_os = "macos"))]
        fn on_before_child_process_launch(&self, command_line: Option<&mut CommandLine>) {
            let supervision = self.supervision.clone();
            let runtime = self.runtime.clone();
            let app = self.pump.app().clone();
            super::ffi_guard::unit_or(
                || {
                    let _ = runtime.mark_failed();
                    crate::services::e2e_profile::report_browser_exit_source(
                        crate::services::e2e_profile::BrowserExitSource::LaunchCallback,
                    );
                    crate::app_exit::request(&app, 1);
                },
                || {
                    if supervision.attach_launch_marker(command_line).is_err() {
                        let _ = runtime.mark_failed();
                        crate::services::e2e_profile::report_browser_exit_source(
                            crate::services::e2e_profile::BrowserExitSource::ChildAdmission,
                        );
                        crate::app_exit::request(&app, 1);
                    }
                },
            );
        }
    }
}
