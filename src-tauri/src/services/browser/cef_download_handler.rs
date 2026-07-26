use super::cef_blocked_feature::BlockedFeatureNotifier;
use cef::*;

cef::wrap_download_handler! {
    pub(super) struct BrowserDownloadHandler {
        notifier: BlockedFeatureNotifier,
    }

    impl DownloadHandler {
        fn can_download(
            &self,
            _browser: Option<&mut Browser>,
            _url: Option<&CefString>,
            _request_method: Option<&CefString>,
        ) -> std::os::raw::c_int {
            super::ffi_guard::value(0, || {
                self.notifier.publish_once();
                0
            })
        }

        fn on_download_updated(
            &self,
            _browser: Option<&mut Browser>,
            _download_item: Option<&mut DownloadItem>,
            callback: Option<&mut DownloadItemCallback>,
        ) {
            super::ffi_guard::unit(|| {
                if let Some(callback) = callback {
                    callback.cancel();
                }
                self.notifier.publish_once();
            });
        }
    }
}
