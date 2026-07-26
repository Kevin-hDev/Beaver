use super::{
    local_site_policy::is_allowed_local_url,
    local_site_types::{LocalSite, LocalSiteScanResult, MAX_LOCAL_RESULTS, MAX_LOCAL_TITLE_CHARS},
};

#[derive(Default)]
pub(super) struct LocalSiteScanState {
    tracked: Vec<TrackedSite>,
    generation: u64,
}

struct TrackedSite {
    site: LocalSite,
    failures: u8,
}

impl LocalSiteScanState {
    pub(super) fn snapshot(&self) -> LocalSiteScanResult {
        LocalSiteScanResult {
            sites: self.sites(),
            generation: self.generation,
            changed: false,
        }
    }

    pub(super) fn merge(
        &mut self,
        mut observed: Vec<LocalSite>,
    ) -> Result<LocalSiteScanResult, ()> {
        validate_observed(&observed)?;
        observed.sort_by_key(|site| site.port);
        let previous = self.sites();
        for tracked in &mut self.tracked {
            if let Some(index) = observed
                .iter()
                .position(|site| site.port == tracked.site.port)
            {
                tracked.site = observed.remove(index);
                tracked.failures = 0;
            } else {
                tracked.failures = tracked.failures.saturating_add(1);
            }
        }
        self.tracked.retain(|tracked| tracked.failures < 2);
        for site in observed {
            if self.tracked.len() == MAX_LOCAL_RESULTS {
                break;
            }
            self.tracked.push(TrackedSite { site, failures: 0 });
        }
        self.tracked.sort_by_key(|tracked| tracked.site.port);
        let sites = self.sites();
        let changed = sites != previous;
        if changed {
            self.generation = self.generation.checked_add(1).ok_or(())?;
        }
        Ok(LocalSiteScanResult {
            sites,
            generation: self.generation,
            changed,
        })
    }

    fn sites(&self) -> Vec<LocalSite> {
        self.tracked
            .iter()
            .map(|tracked| tracked.site.clone())
            .collect()
    }
}

fn validate_observed(sites: &[LocalSite]) -> Result<(), ()> {
    if sites.len() > MAX_LOCAL_RESULTS {
        return Err(());
    }
    for (index, site) in sites.iter().enumerate() {
        let url = url::Url::parse(&site.url).map_err(|_| ())?;
        if site.port == 0
            || site.title.chars().count() > MAX_LOCAL_TITLE_CHARS
            || !is_allowed_local_url(&url)
            || sites[..index]
                .iter()
                .any(|previous| previous.port == site.port)
        {
            return Err(());
        }
    }
    Ok(())
}
