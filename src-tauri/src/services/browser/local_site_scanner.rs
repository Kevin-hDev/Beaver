use super::{
    local_site_candidates::listening_candidates,
    local_site_probe::probe_candidate,
    local_site_scan_state::LocalSiteScanState,
    local_site_scan_throttle::LocalSiteScanThrottle,
    local_site_types::{LocalSite, LocalSiteScanResult, MAX_LOCAL_RESULTS},
};
use futures_util::{stream, StreamExt};
use std::sync::Arc;
use std::time::Instant;

const MAX_CONCURRENT_PROBES: usize = 8;

#[derive(Clone, Default)]
pub struct LocalSiteScanner {
    gate: Arc<tokio::sync::Mutex<()>>,
    throttle: Arc<tokio::sync::Mutex<LocalSiteScanThrottle>>,
    state: Arc<tokio::sync::Mutex<LocalSiteScanState>>,
}

impl LocalSiteScanner {
    pub async fn scan(&self, home_visible: bool) -> Result<LocalSiteScanResult, ()> {
        if !home_visible {
            return Ok(self.state.lock().await.snapshot());
        }
        let _guard = self.gate.lock().await;
        if !self.throttle.lock().await.allow(Instant::now()) {
            return Ok(self.state.lock().await.snapshot());
        }
        let candidates = tokio::task::spawn_blocking(listening_candidates)
            .await
            .map_err(|_| ())??;
        let mut sites: Vec<LocalSite> = stream::iter(candidates)
            .map(probe_candidate)
            .buffer_unordered(MAX_CONCURRENT_PROBES)
            .filter_map(|result| async move { result.ok() })
            .collect()
            .await;
        sites.sort_by_key(|site| site.port);
        sites.truncate(MAX_LOCAL_RESULTS);
        self.state.lock().await.merge(sites)
    }
}
