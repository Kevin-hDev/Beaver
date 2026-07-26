use super::cef_blocked_feature::BlockedFeatureNotifier;
use cef::*;

cef::wrap_permission_handler! {
    pub(super) struct BrowserPermissionHandler {
        notifier: BlockedFeatureNotifier,
    }

    impl PermissionHandler {
        fn on_request_media_access_permission(
            &self,
            _browser: Option<&mut Browser>,
            _frame: Option<&mut Frame>,
            _requesting_origin: Option<&CefString>,
            _requested_permissions: u32,
            callback: Option<&mut MediaAccessCallback>,
        ) -> std::os::raw::c_int {
            super::ffi_guard::value(1, || {
                if let Some(callback) = callback {
                    callback.cancel();
                }
                self.notifier.publish_once();
                1
            })
        }

        fn on_show_permission_prompt(
            &self,
            _browser: Option<&mut Browser>,
            _prompt_id: u64,
            _requesting_origin: Option<&CefString>,
            _requested_permissions: u32,
            callback: Option<&mut PermissionPromptCallback>,
        ) -> std::os::raw::c_int {
            super::ffi_guard::value(1, || {
                if let Some(callback) = callback {
                    callback.cont(PermissionRequestResult::DENY);
                }
                self.notifier.publish_once();
                1
            })
        }
    }
}
