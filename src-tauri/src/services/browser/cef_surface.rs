use super::{
    browser_api_types::BrowserNavigationAction,
    browser_view_key::BrowserViewKey,
    cef_state_bridge::mark_view_released,
    cef_surface_view::CefBrowserView,
    surface_bounds::{BrowserSurfaceBounds, SurfaceTracker, SurfaceUpdate},
    url_policy::ValidatedUrl,
    view_recency::ViewRecency,
};

pub(super) struct BrowserSurfaceManager {
    views: Vec<CefBrowserView>,
    recency: ViewRecency,
    active: Option<BrowserViewKey>,
    tracked_key: Option<BrowserViewKey>,
    tracker: SurfaceTracker,
}

impl BrowserSurfaceManager {
    pub(super) fn new() -> Self {
        Self {
            views: Vec::with_capacity(super::session_types::MAX_BROWSER_TABS),
            recency: ViewRecency::default(),
            active: None,
            tracked_key: None,
            tracker: SurfaceTracker::default(),
        }
    }

    pub(super) fn apply(
        &mut self,
        app: &tauri::AppHandle,
        key: BrowserViewKey,
        url: Option<ValidatedUrl>,
        bounds: BrowserSurfaceBounds,
    ) -> Result<(), ()> {
        bounds.validate()?;
        if self.tracked_key.as_ref() != Some(&key) {
            self.tracked_key = Some(key.clone());
            self.tracker = SurfaceTracker::default();
        }
        if self.tracker.classify(bounds.clone()) == SurfaceUpdate::Stale {
            return Ok(());
        }
        let Some(url) = url.filter(|_| bounds.visible) else {
            self.hide_all(app, &bounds)?;
            self.active = None;
            return Ok(());
        };
        self.hide_all_except(app, &key, &bounds)?;
        if let Some(view) = self.views.iter_mut().find(|view| view.key() == &key) {
            view.apply(app, &url, &bounds)?;
            self.recency.touch(key.clone(), None);
            self.active = Some(key);
            return Ok(());
        }
        if let Some(evicted) = self.recency.touch(key.clone(), None) {
            self.evict(app, &evicted);
        }
        match CefBrowserView::create(app, key.clone(), &url, &bounds) {
            Ok(view) => self.views.push(view),
            Err(()) => {
                self.recency.remove(&key);
                return Err(());
            }
        }
        self.active = Some(key);
        Ok(())
    }

    pub(super) fn action(
        &mut self,
        key: &BrowserViewKey,
        action: BrowserNavigationAction,
    ) -> Result<(), ()> {
        self.view_mut(key)?.action(action)
    }

    pub(super) fn navigate(&mut self, key: &BrowserViewKey, url: &ValidatedUrl) -> Result<(), ()> {
        let Some(view) = self.views.iter_mut().find(|view| view.key() == key) else {
            return Ok(());
        };
        view.navigate(url)
    }

    pub(super) fn close_view(&mut self, app: &tauri::AppHandle, key: &BrowserViewKey) {
        if let Some(index) = self.views.iter().position(|view| view.key() == key) {
            let mut view = self.views.remove(index);
            let _ = view.close(Some(app));
        }
        self.recency.remove(key);
        if self.active.as_ref() == Some(key) {
            self.active = None;
        }
    }

    pub(super) fn reset_page_surface(&mut self, app: &tauri::AppHandle) -> Result<(), ()> {
        let mut failed = false;
        for view in &mut self.views {
            failed |= view.hide_current(app).is_err();
        }
        self.active = None;
        self.tracked_key = None;
        self.tracker.reset();
        (!failed).then_some(()).ok_or(())
    }

    pub(super) fn close(&mut self) {
        for view in &mut self.views {
            let _ = view.close(None);
        }
        self.views.clear();
        self.active = None;
    }

    fn hide_all_except(
        &mut self,
        app: &tauri::AppHandle,
        active: &BrowserViewKey,
        bounds: &BrowserSurfaceBounds,
    ) -> Result<(), ()> {
        let mut hidden = bounds.clone();
        hidden.visible = false;
        let mut failed = false;
        for view in self.views.iter_mut().filter(|view| view.key() != active) {
            failed |= view.hide(app, &hidden).is_err();
        }
        (!failed).then_some(()).ok_or(())
    }

    fn hide_all(
        &mut self,
        app: &tauri::AppHandle,
        bounds: &BrowserSurfaceBounds,
    ) -> Result<(), ()> {
        let mut hidden = bounds.clone();
        hidden.visible = false;
        let mut failed = false;
        for view in &mut self.views {
            failed |= view.hide(app, &hidden).is_err();
        }
        (!failed).then_some(()).ok_or(())
    }

    fn evict(&mut self, app: &tauri::AppHandle, key: &BrowserViewKey) {
        if let Some(index) = self.views.iter().position(|view| view.key() == key) {
            let mut view = self.views.remove(index);
            if let Some(stamp) = view.close(Some(app)) {
                mark_view_released(app, key.clone(), stamp);
            }
        }
    }

    fn view_mut(&mut self, key: &BrowserViewKey) -> Result<&mut CefBrowserView, ()> {
        self.views
            .iter_mut()
            .find(|view| view.key() == key)
            .ok_or(())
    }
}
