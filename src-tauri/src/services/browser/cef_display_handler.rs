use super::{
    browser_slot::BrowserSlot,
    browser_view_key::BrowserViewKey,
    cef_state_bridge::submit_runtime_update,
    cef_text::{bounded_cef_text, validated_cef_url},
    session_types::BrowserRuntimeTabUpdate,
};
use cef::*;

cef::wrap_display_handler! {
    pub(super) struct BrowserDisplayHandler {
        slot: BrowserSlot,
        app: tauri::AppHandle,
        key: BrowserViewKey,
    }

    impl DisplayHandler {
        fn on_favicon_urlchange(&self, _browser: Option<&mut Browser>, icon_urls: Option<&mut CefStringList>) {
            super::ffi_guard::unit(|| {
                let Some(epoch) = self.slot.epoch() else { return; };
                let urls = icon_urls.as_deref().map(super::cef_favicon_candidates::read).unwrap_or_default();
                super::favicon_runtime::mutate(&self.app, |state| state.replace_candidates(&self.key, epoch, urls));
                super::cef_favicon_scheduler::schedule(&self.app);
            });
        }

        fn on_address_change(
            &self,
            _browser: Option<&mut Browser>,
            frame: Option<&mut Frame>,
            url: Option<&CefString>,
        ) {
            super::ffi_guard::unit(|| {
                if frame.is_none_or(|frame| frame.is_main() != 1) {
                    return;
                }
                let Some(url) = url.and_then(validated_cef_url) else {
                    return;
                };
                let Some(stamp) = self.slot.next_runtime_stamp() else {
                    return;
                };
                self.slot.observe_url(&url);
                submit_runtime_update(
                    &self.app,
                    self.key.clone(),
                    stamp,
                    BrowserRuntimeTabUpdate { url: Some(url), ..Default::default() },
                );
            });
        }

        fn on_title_change(&self, _browser: Option<&mut Browser>, title: Option<&CefString>) {
            super::ffi_guard::unit(|| {
                let Some(title) = title.and_then(|value| bounded_cef_text(value, 256)) else {
                    return;
                };
                let Some(stamp) = self.slot.next_runtime_stamp() else {
                    return;
                };
                submit_runtime_update(
                    &self.app,
                    self.key.clone(),
                    stamp,
                    BrowserRuntimeTabUpdate { title: Some(title), ..Default::default() },
                );
            });
        }
    }
}
