use super::{
    browser_view_key::BrowserViewKey,
    favicon_policy::{self, DOWNLOAD_DEADLINE, MAX_DOWNLOADS, MAX_FAVICONS, MAX_REVISION},
    favicon_types::{FaviconEntry, FaviconJob, FaviconTicket},
};
use std::time::Instant;

#[derive(Default)]
pub(super) struct FaviconState {
    pub(super) entries: Vec<FaviconEntry>,
    pub(super) revision: u64,
    // Permits outlive evicted views and logical deadlines: neither cancels CEF.
    running: Vec<FaviconJob>,
}

impl FaviconState {
    fn next_revision(&mut self) -> Option<u64> {
        self.revision = self.revision.checked_add(1)?.min(MAX_REVISION);
        (self.revision < MAX_REVISION).then_some(self.revision)
    }

    pub(super) fn begin_document(&mut self, key: BrowserViewKey, epoch: u64) {
        if self
            .entries
            .iter()
            .any(|e| e.key == key && e.ticket.view_epoch > epoch)
        {
            return;
        }
        self.entries.retain(|e| e.key != key);
        let Some(document) = self.next_revision() else {
            return;
        };
        if self.entries.len() == MAX_FAVICONS {
            self.entries.remove(0);
        }
        self.entries.push(FaviconEntry {
            key,
            ticket: FaviconTicket {
                view_epoch: epoch,
                document,
                request: document,
            },
            candidates: Vec::new(),
            png: None,
        });
    }

    pub(super) fn replace_candidates(
        &mut self,
        key: &BrowserViewKey,
        epoch: u64,
        urls: Vec<String>,
    ) {
        let Some(index) = self
            .entries
            .iter()
            .position(|e| &e.key == key && e.ticket.view_epoch == epoch)
        else {
            return;
        };
        let mut entry = self.entries.remove(index);
        let Some(request) = self.next_revision() else {
            return;
        };
        entry.ticket.request = request;
        entry.candidates = favicon_policy::candidates(urls);
        entry.png = None;
        self.entries.push(entry);
    }

    pub(super) fn release_view(&mut self, key: &BrowserViewKey, epoch: u64) {
        let previous = self.entries.len();
        self.entries
            .retain(|e| &e.key != key || e.ticket.view_epoch != epoch);
        if self.entries.len() != previous {
            self.next_revision();
        }
    }

    pub(super) fn take_ready(&mut self, now: Instant) -> Vec<FaviconJob> {
        let mut jobs = Vec::new();
        if self.revision >= MAX_REVISION {
            return jobs;
        }
        for entry in &mut self.entries {
            if self.running.len() == MAX_DOWNLOADS {
                break;
            }
            if entry.candidates.is_empty() || self.running.iter().any(|j| j.key == entry.key) {
                continue;
            }
            let job = FaviconJob {
                key: entry.key.clone(),
                ticket: entry.ticket,
                url: entry.candidates.remove(0),
                started: now,
            };
            self.running.push(job.clone());
            jobs.push(job);
        }
        jobs
    }

    pub(super) fn is_current(&self, job: &FaviconJob, now: Instant) -> bool {
        self.revision < MAX_REVISION
            && now.saturating_duration_since(job.started) < DOWNLOAD_DEADLINE
            && self
                .entries
                .iter()
                .any(|e| e.key == job.key && e.ticket == job.ticket)
    }

    pub(super) fn complete(&mut self, job: &FaviconJob, png: Option<String>, now: Instant) {
        if !self.is_current(job, now) {
            return;
        }
        if let Some(png) = png {
            let Some(revision) = self.next_revision() else {
                return;
            };
            if let Some(entry) = self.entries.iter_mut().find(|e| e.key == job.key) {
                entry.png = Some(png);
                entry.candidates.clear();
                // Invalidate duplicate deliveries without changing the document identity.
                entry.ticket.request = revision;
            }
        }
    }

    pub(super) fn finish_callback(&mut self, job: &FaviconJob) {
        self.running
            .retain(|j| j.ticket != job.ticket || j.key != job.key || j.url != job.url);
    }
}
