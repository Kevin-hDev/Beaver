use super::{
    browser_events::{
        next_event_generation, BrowserTabEvent, PopupRequestEvent, EVENT_VERSION,
        POPUP_REQUEST_EVENT, VIEW_READY_EVENT,
    },
    browser_slot::BrowserSlot,
    browser_view_key::BrowserViewKey,
    cef_text::validated_cef_url,
    native_surface,
};
use cef::*;
use tauri::Emitter;

cef::wrap_life_span_handler! {
    pub(super) struct BrowserLifeSpanHandler {
        slot: BrowserSlot,
        app: tauri::AppHandle,
        key: BrowserViewKey,
    }

    impl LifeSpanHandler {
        fn on_before_popup(
            &self,
            _browser: Option<&mut Browser>,
            _frame: Option<&mut Frame>,
            _popup_id: std::os::raw::c_int,
            target_url: Option<&CefString>,
            _target_frame_name: Option<&CefString>,
            _target_disposition: WindowOpenDisposition,
            _user_gesture: std::os::raw::c_int,
            _popup_features: Option<&PopupFeatures>,
            _window_info: Option<&mut WindowInfo>,
            _client: Option<&mut Option<Client>>,
            _settings: Option<&mut BrowserSettings>,
            _extra_info: Option<&mut Option<DictionaryValue>>,
            _no_javascript_access: Option<&mut std::os::raw::c_int>,
        ) -> std::os::raw::c_int {
            super::ffi_guard::value(1, || {
                if let (Some(url), Some(generation)) = (
                    target_url.and_then(validated_cef_url),
                    next_event_generation(),
                ) {
                    let _ = self.app.emit(
                        POPUP_REQUEST_EVENT,
                        PopupRequestEvent {
                            event_version: EVENT_VERSION,
                            generation,
                            conversation_id: self.key.session_id.clone(),
                            source_tab_id: self.key.tab_id.clone(),
                            url,
                        },
                    );
                }
                1
            })
        }

        fn on_after_created(&self, browser: Option<&mut Browser>) {
            super::ffi_guard::unit(|| {
                let Some(browser) = browser else {
                    return;
                };
                let Some(bounds) = self.slot.mark_created(browser) else {
                    close_browser(browser);
                    return;
                };
                if native_surface::update_browser(&self.app, browser, &bounds).is_err() {
                    close_browser(browser);
                    return;
                }
                let Some(generation) = next_event_generation() else {
                    close_browser(browser);
                    return;
                };
                let _ = self.app.emit(
                    VIEW_READY_EVENT,
                    BrowserTabEvent {
                        event_version: EVENT_VERSION,
                        generation,
                        conversation_id: self.key.session_id.clone(),
                        tab_id: self.key.tab_id.clone(),
                    },
                );
            });
        }

        fn do_close(&self, browser: Option<&mut Browser>) -> std::os::raw::c_int {
            super::ffi_guard::value(1, || {
                if let Some(browser) = browser {
                    let _ = native_surface::destroy_browser(browser);
                }
                1
            })
        }

        fn on_before_close(&self, _browser: Option<&mut Browser>) {
            super::ffi_guard::unit(|| self.slot.mark_closed());
        }
    }
}

fn close_browser(browser: &Browser) {
    if let Some(host) = browser.host() {
        host.close_browser(1);
    }
}
