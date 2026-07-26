use super::{
    browser_slot::BrowserSlot, browser_view_key::BrowserViewKey,
    cef_state_bridge::submit_runtime_update, session_types::BrowserRuntimeTabUpdate,
};
use cef::*;

cef::wrap_load_handler! {
    pub(super) struct BrowserLoadHandler {
        slot: BrowserSlot,
        app: tauri::AppHandle,
        key: BrowserViewKey,
    }

    impl LoadHandler {
        fn on_loading_state_change(
            &self,
            _browser: Option<&mut Browser>,
            is_loading: std::os::raw::c_int,
            can_go_back: std::os::raw::c_int,
            can_go_forward: std::os::raw::c_int,
        ) {
            super::ffi_guard::unit(|| {
                let Some(stamp) = self.slot.next_runtime_stamp() else {
                    return;
                };
                submit_runtime_update(
                    &self.app,
                    self.key.clone(),
                    stamp,
                    BrowserRuntimeTabUpdate {
                        loading: Some(is_loading == 1),
                        can_go_back: Some(can_go_back == 1),
                        can_go_forward: Some(can_go_forward == 1),
                        ..Default::default()
                    },
                );
            });
        }
    }
}
