use super::{
    browser_slot::BrowserSlot, browser_view_key::BrowserViewKey,
    cef_blocked_feature::BlockedFeatureNotifier, cef_display_handler::BrowserDisplayHandler,
    cef_download_handler::BrowserDownloadHandler, cef_life_span_handler::BrowserLifeSpanHandler,
    cef_load_handler::BrowserLoadHandler, cef_permission_handler::BrowserPermissionHandler,
    cef_request_handler::BrowserRequestHandler,
};
use cef::*;

cef::wrap_client! {
    pub(super) struct BrowserClient {
        display_handler: DisplayHandler,
        download_handler: DownloadHandler,
        life_span_handler: LifeSpanHandler,
        load_handler: LoadHandler,
        permission_handler: PermissionHandler,
        request_handler: RequestHandler,
    }

    impl Client {
        fn display_handler(&self) -> Option<DisplayHandler> {
            super::ffi_guard::value(None, || Some(self.display_handler.clone()))
        }

        fn download_handler(&self) -> Option<DownloadHandler> {
            super::ffi_guard::value(None, || Some(self.download_handler.clone()))
        }

        fn life_span_handler(&self) -> Option<LifeSpanHandler> {
            super::ffi_guard::value(None, || Some(self.life_span_handler.clone()))
        }

        fn request_handler(&self) -> Option<RequestHandler> {
            super::ffi_guard::value(None, || Some(self.request_handler.clone()))
        }

        fn load_handler(&self) -> Option<LoadHandler> {
            super::ffi_guard::value(None, || Some(self.load_handler.clone()))
        }

        fn permission_handler(&self) -> Option<PermissionHandler> {
            super::ffi_guard::value(None, || Some(self.permission_handler.clone()))
        }
    }
}

pub(super) fn create_browser_client(
    slot: BrowserSlot,
    app: tauri::AppHandle,
    key: BrowserViewKey,
) -> Client {
    let download_notifier = BlockedFeatureNotifier::new(app.clone(), key.clone());
    let permission_notifier = BlockedFeatureNotifier::new(app.clone(), key.clone());
    BrowserClient::new(
        BrowserDisplayHandler::new(slot.clone(), app.clone(), key.clone()),
        BrowserDownloadHandler::new(download_notifier),
        BrowserLifeSpanHandler::new(slot.clone(), app.clone(), key.clone()),
        BrowserLoadHandler::new(slot.clone(), app.clone(), key.clone()),
        BrowserPermissionHandler::new(permission_notifier),
        BrowserRequestHandler::new(slot, app, key),
    )
}
