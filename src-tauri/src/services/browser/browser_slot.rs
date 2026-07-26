use super::{
    navigation_target::NavigationTarget, runtime_revision::RuntimeStamp,
    surface_bounds::BrowserSurfaceBounds, view_state::ViewState,
};
use cef::{Browser, CefString, ImplBrowser, ImplBrowserHost, ImplFrame};
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc, Mutex,
};

static NEXT_RUNTIME_EPOCH: AtomicU64 = AtomicU64::new(1);

#[derive(Clone)]
pub(super) struct BrowserSlot {
    inner: Arc<Mutex<BrowserSlotInner>>,
}

struct BrowserSlotInner {
    browser: Option<Browser>,
    lifecycle: ViewState,
    navigation: NavigationTarget,
    desired_surface: Option<BrowserSurfaceBounds>,
    runtime_epoch: u64,
    runtime_revision: u64,
}

impl BrowserSlot {
    pub(super) fn new() -> Option<Self> {
        let runtime_epoch = NEXT_RUNTIME_EPOCH
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |current| {
                current.checked_add(1)
            })
            .ok()?;
        Some(Self {
            inner: Arc::new(Mutex::new(BrowserSlotInner {
                browser: None,
                lifecycle: ViewState::default(),
                navigation: NavigationTarget::default(),
                desired_surface: None,
                runtime_epoch,
                runtime_revision: 0,
            })),
        })
    }

    pub(super) fn begin_creation(&self) -> bool {
        self.inner
            .lock()
            .is_ok_and(|mut inner| inner.lifecycle.begin_creation())
    }

    pub(super) fn mark_creation_failed(&self) {
        if let Ok(mut inner) = self.inner.lock() {
            let _ = inner.lifecycle.mark_creation_failed();
        }
    }

    pub(super) fn mark_created(&self, browser: &Browser) -> Option<BrowserSurfaceBounds> {
        let (pending, surface) = {
            let Ok(mut inner) = self.inner.lock() else {
                return None;
            };
            if !inner.lifecycle.mark_ready() {
                return None;
            }
            inner.browser = Some(browser.clone());
            (
                inner.navigation.take_pending(),
                inner.desired_surface.clone(),
            )
        };
        if let Some(url) = pending {
            if let Some(frame) = browser.main_frame() {
                frame.load_url(Some(&CefString::from(url.as_str())));
            }
        }
        surface
    }

    pub(super) fn request_surface(
        &self,
        bounds: &BrowserSurfaceBounds,
    ) -> Result<Option<Browser>, ()> {
        let mut inner = self.inner.lock().map_err(|_| ())?;
        inner.desired_surface = Some(bounds.clone());
        Ok(inner.browser.clone())
    }

    #[cfg(test)]
    pub(super) fn desired_surface(&self) -> Option<BrowserSurfaceBounds> {
        self.inner
            .lock()
            .ok()
            .and_then(|inner| inner.desired_surface.clone())
    }

    pub(super) fn browser(&self) -> Option<Browser> {
        self.inner
            .lock()
            .ok()
            .and_then(|inner| inner.browser.clone())
    }

    pub(super) fn navigate(&self, url: &str) -> bool {
        let browser = {
            let Ok(mut inner) = self.inner.lock() else {
                return false;
            };
            inner.navigation.request(url);
            inner.browser.clone()
        };
        let Some(browser) = browser else {
            return true;
        };
        let Some(frame) = browser.main_frame() else {
            return false;
        };
        frame.load_url(Some(&CefString::from(url)));
        true
    }

    pub(super) fn observe_url(&self, url: &str) {
        if let Ok(mut inner) = self.inner.lock() {
            inner.navigation.observe(url);
        }
    }

    pub(super) fn next_runtime_stamp(&self) -> Option<RuntimeStamp> {
        let mut inner = self.inner.lock().ok()?;
        if !inner.lifecycle.is_ready() {
            return None;
        }
        inner.runtime_revision = inner.runtime_revision.checked_add(1)?;
        RuntimeStamp::new(inner.runtime_epoch, inner.runtime_revision)
    }

    pub(super) fn close(&self) {
        let browser = self.inner.lock().ok().and_then(|mut inner| {
            if let Some(surface) = inner.desired_surface.as_mut() {
                surface.visible = false;
            }
            inner
                .lifecycle
                .begin_closing()
                .then(|| inner.browser.clone())
        });
        if let Some(Some(browser)) = browser {
            if let Some(host) = browser.host() {
                host.close_browser(1);
            }
        }
    }

    pub(super) fn hide_requested_surface(
        &self,
    ) -> Result<Option<(Browser, BrowserSurfaceBounds)>, ()> {
        let mut inner = self.inner.lock().map_err(|_| ())?;
        let Some(surface) = inner.desired_surface.as_mut() else {
            return Ok(None);
        };
        surface.visible = false;
        let hidden = surface.clone();
        Ok(inner.browser.clone().map(|browser| (browser, hidden)))
    }

    pub(super) fn mark_closed(&self) {
        if let Ok(mut inner) = self.inner.lock() {
            let _ = inner.lifecycle.begin_closing();
            inner.browser = None;
            let _ = inner.lifecycle.mark_closed();
        }
    }
}
