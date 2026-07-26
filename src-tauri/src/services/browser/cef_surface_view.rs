use super::{
    browser_api_types::BrowserNavigationAction, browser_slot::BrowserSlot,
    browser_view_key::BrowserViewKey, cef_client::create_browser_client, native_surface,
    surface_bounds::BrowserSurfaceBounds, url_policy::ValidatedUrl,
};
use cef::*;

pub(super) struct CefBrowserView {
    key: BrowserViewKey,
    slot: BrowserSlot,
    client: Option<Client>,
}

impl CefBrowserView {
    pub(super) fn create(
        app: &tauri::AppHandle,
        key: BrowserViewKey,
        url: &ValidatedUrl,
        bounds: &BrowserSurfaceBounds,
    ) -> Result<Self, ()> {
        let slot = BrowserSlot::new().ok_or(())?;
        if !slot.begin_creation() {
            return Err(());
        }
        slot.request_surface(bounds).inspect_err(|_| {
            slot.mark_creation_failed();
        })?;
        slot.observe_url(url.as_str());
        let parent = native_surface::resolve_parent(app, bounds).inspect_err(|_| {
            slot.mark_creation_failed();
        })?;
        let mut client = create_browser_client(slot.clone(), app.clone(), key.clone());
        let created = browser_host_create_browser(
            Some(&parent.window_info()),
            Some(&mut client),
            Some(&CefString::from(url.as_str())),
            Some(&secure_browser_settings()),
            None,
            None,
        );
        if created != 1 {
            slot.mark_creation_failed();
            return Err(());
        }
        Ok(Self {
            key,
            slot,
            client: Some(client),
        })
    }

    pub(super) fn key(&self) -> &BrowserViewKey {
        &self.key
    }

    pub(super) fn apply(
        &mut self,
        app: &tauri::AppHandle,
        _url: &ValidatedUrl,
        bounds: &BrowserSurfaceBounds,
    ) -> Result<(), ()> {
        let Some(browser) = self.slot.request_surface(bounds)? else {
            return Ok(());
        };
        native_surface::update_browser(app, &browser, bounds)?;
        Ok(())
    }

    pub(super) fn navigate(&mut self, url: &ValidatedUrl) -> Result<(), ()> {
        self.slot.navigate(url.as_str()).then_some(()).ok_or(())
    }

    pub(super) fn hide(
        &mut self,
        app: &tauri::AppHandle,
        bounds: &BrowserSurfaceBounds,
    ) -> Result<(), ()> {
        if let Some(browser) = self.slot.request_surface(bounds)? {
            native_surface::update_browser(app, &browser, bounds)?;
        }
        Ok(())
    }

    pub(super) fn hide_current(&mut self, app: &tauri::AppHandle) -> Result<(), ()> {
        if let Some((browser, bounds)) = self.slot.hide_requested_surface()? {
            native_surface::update_browser(app, &browser, &bounds)?;
        }
        Ok(())
    }

    pub(super) fn action(&mut self, action: BrowserNavigationAction) -> Result<(), ()> {
        let browser = self.slot.browser().ok_or(())?;
        match action {
            BrowserNavigationAction::Back if browser.can_go_back() == 1 => browser.go_back(),
            BrowserNavigationAction::Forward if browser.can_go_forward() == 1 => {
                browser.go_forward();
            }
            BrowserNavigationAction::ReloadOrStop if browser.is_loading() == 1 => {
                browser.stop_load();
            }
            BrowserNavigationAction::ReloadOrStop => browser.reload(),
            _ => {}
        }
        Ok(())
    }

    pub(super) fn close(
        &mut self,
        app: Option<&tauri::AppHandle>,
    ) -> Option<super::runtime_revision::RuntimeStamp> {
        let stamp = self.slot.next_runtime_stamp();
        if let Some(app) = app {
            let _ = self.hide_current(app);
        }
        self.slot.close();
        self.client = None;
        stamp
    }
}

fn secure_browser_settings() -> BrowserSettings {
    BrowserSettings {
        javascript_close_windows: State::DISABLED,
        javascript_access_clipboard: State::DISABLED,
        javascript_dom_paste: State::DISABLED,
        local_storage: State::ENABLED,
        ..Default::default()
    }
}
