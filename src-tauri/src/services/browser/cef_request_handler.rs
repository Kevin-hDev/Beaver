use super::{
    browser_events::{next_event_generation, BrowserTabEvent, ENGINE_STOPPED_EVENT, EVENT_VERSION},
    browser_slot::BrowserSlot,
    browser_view_key::BrowserViewKey,
    cef_state_bridge::mark_view_released,
    cef_text::validated_cef_url,
};
use cef::*;
use tauri::Emitter;

cef::wrap_request_handler! {
    pub(super) struct BrowserRequestHandler {
        slot: BrowserSlot,
        app: tauri::AppHandle,
        key: BrowserViewKey,
    }

    impl RequestHandler {
        fn on_before_browse(
            &self,
            _browser: Option<&mut Browser>,
            _frame: Option<&mut Frame>,
            request: Option<&mut Request>,
            _user_gesture: std::os::raw::c_int,
            _is_redirect: std::os::raw::c_int,
        ) -> std::os::raw::c_int {
            super::ffi_guard::value(1, || {
                let allowed = request.is_some_and(|request| {
                    let raw = request.url();
                    let value = CefString::from(&raw);
                    validated_cef_url(&value).is_some()
                });
                i32::from(!allowed)
            })
        }

        fn on_certificate_error(
            &self,
            _browser: Option<&mut Browser>,
            _cert_error: Errorcode,
            _request_url: Option<&CefString>,
            _ssl_info: Option<&mut Sslinfo>,
            _callback: Option<&mut Callback>,
        ) -> std::os::raw::c_int {
            super::ffi_guard::value(0, || 0)
        }

        fn on_render_process_terminated(
            &self,
            _browser: Option<&mut Browser>,
            status: TerminationStatus,
            error_code: std::os::raw::c_int,
            _error_string: Option<&CefString>,
        ) {
            super::ffi_guard::unit(|| {
                super::cef_diagnostics::log_renderer_termination(status, error_code);
                let key = self.key.clone();
                let app = self.app.clone();
                if let Some(stamp) = self.slot.next_runtime_stamp() {
                    mark_view_released(&app, key.clone(), stamp);
                }
                let main_app = app.clone();
                let _ = app.run_on_main_thread(move || {
                    let _ = super::cef_engine::close_view(&main_app, &key);
                });
                if let Some(generation) = next_event_generation() {
                    let _ = self.app.emit(
                        ENGINE_STOPPED_EVENT,
                        BrowserTabEvent {
                            event_version: EVENT_VERSION,
                            generation,
                            conversation_id: self.key.session_id.clone(),
                            tab_id: self.key.tab_id.clone(),
                        },
                    );
                }
            });
        }
    }
}
