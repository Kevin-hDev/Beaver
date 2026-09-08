use super::browser_view_key::BrowserViewKey;
use std::time::Instant;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct FaviconTicket {
    pub view_epoch: u64,
    pub document: u64,
    pub request: u64,
}

pub(super) struct FaviconEntry {
    pub key: BrowserViewKey,
    pub ticket: FaviconTicket,
    pub candidates: Vec<String>,
    pub png: Option<String>,
}

#[derive(Clone)]
pub(super) struct FaviconJob {
    pub key: BrowserViewKey,
    pub ticket: FaviconTicket,
    pub url: String,
    pub started: Instant,
}
