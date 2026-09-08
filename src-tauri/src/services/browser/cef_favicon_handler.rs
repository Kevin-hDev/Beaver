use super::{
    cef_favicon_image, cef_favicon_scheduler,
    favicon_policy::REQUEST_DIP,
    favicon_runtime::{access, mutate},
    favicon_types::FaviconJob,
};
use cef::*;
use std::{sync::Arc, time::Instant};

struct DownloadPermit {
    app: tauri::AppHandle,
    job: FaviconJob,
    _watchdog: super::favicon_watchdog::DownloadWatchdog,
}

impl Drop for DownloadPermit {
    fn drop(&mut self) {
        // Destruction is an actual native lifetime boundary; elapsed time is not.
        access(Some(&self.app), |state| state.finish_callback(&self.job));
        cef_favicon_scheduler::schedule(&self.app);
    }
}

cef::wrap_download_image_callback! {
    struct FaviconDownload { permit: Arc<DownloadPermit> }
    impl DownloadImageCallback {
        fn on_download_image_finished(
            &self,
            _image_url: Option<&CefString>,
            http_status_code: std::os::raw::c_int,
            image: Option<&mut Image>,
        ) {
            super::ffi_guard::unit(|| {
                let permit = &self.permit;
                if !access(Some(&permit.app), |state| state.is_current(&permit.job, Instant::now())) {
                    return;
                }
                let png = if (200..300).contains(&http_status_code) {
                    image.and_then(|image| cef_favicon_image::png(image))
                } else { None };
                if png.is_none() {
                    log::debug!("[browser] favicon candidate unavailable request={} status={http_status_code}", permit.job.ticket.request);
                }
                mutate(&permit.app, |state| state.complete(&permit.job, png, Instant::now()));
            });
        }
    }
}

pub(super) fn download(app: &tauri::AppHandle, job: FaviconJob, browser: Option<Browser>) {
    let permit = Arc::new(DownloadPermit {
        app: app.clone(),
        _watchdog: super::favicon_watchdog::DownloadWatchdog::start(&job),
        job,
    });
    let Some(host) = browser.and_then(|browser| browser.host()) else {
        return;
    };
    if !access(Some(app), |state| {
        state.is_current(&permit.job, Instant::now())
    }) {
        return;
    }
    let url = CefString::from(permit.job.url.as_str());
    let mut callback = FaviconDownload::new(permit);
    host.download_image(Some(&url), 1, REQUEST_DIP, 0, Some(&mut callback));
}
