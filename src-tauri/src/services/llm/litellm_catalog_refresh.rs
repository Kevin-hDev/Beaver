use super::litellm_catalog::{
    get_lock, is_body_size_ok, is_trusted_host, CatalogParseError, MAX_BODY_BYTES,
};
use futures_util::StreamExt;
use std::io::Read;
use std::path::PathBuf;

const SOURCE_URL: &str =
    "https://raw.githubusercontent.com/BerriAI/litellm/main/model_prices_and_context_window.json";

pub fn cache_path() -> PathBuf {
    crate::services::paths::data_dir().join("litellm-models.json")
}

pub fn read_cache() -> Option<String> {
    let file = std::fs::File::open(cache_path()).ok()?;
    let mut bytes = Vec::new();
    file.take((MAX_BODY_BYTES as u64).saturating_add(1))
        .read_to_end(&mut bytes)
        .ok()?;
    if bytes.len() > MAX_BODY_BYTES {
        return None;
    }
    String::from_utf8(bytes).ok()
}

pub async fn refresh() {
    let client = match reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
    {
        Ok(client) => client,
        Err(_) => {
            log_refresh_rejection("client_build_failed", None, None);
            return;
        }
    };
    let cached = cache_path();
    let mut request = client.get(SOURCE_URL);
    if let Ok(modified) = std::fs::metadata(&cached).and_then(|metadata| metadata.modified()) {
        request = request.header("If-Modified-Since", httpdate::fmt_http_date(modified));
    }
    let response = match request.send().await {
        Ok(response) => response,
        Err(_) => {
            log_refresh_rejection("fetch_failed", None, None);
            return;
        }
    };
    if response.status() == 304 {
        return;
    }
    let status = response.status();
    if let Some(reason) = response_rejection(
        status,
        response.url().host_str().is_some_and(is_trusted_host),
        response.content_length(),
    ) {
        log_refresh_rejection(reason, Some(status.as_u16()), None);
        return;
    }
    let body = match read_body(response).await {
        Ok(body) => body,
        Err(reason) => {
            log_refresh_rejection(reason, Some(status.as_u16()), None);
            return;
        }
    };
    if let Err(rejection) = publish_catalog(&cached, get_lock(), &body).await {
        log_refresh_rejection(
            rejection.reason(),
            Some(status.as_u16()),
            Some(rejection.entries_observed()),
        );
    }
}

pub(super) fn response_rejection(
    status: reqwest::StatusCode,
    trusted_host: bool,
    content_length: Option<u64>,
) -> Option<&'static str> {
    if !status.is_success() {
        return Some("upstream_status");
    }
    if !trusted_host {
        return Some("untrusted_redirect");
    }
    if content_length
        .is_some_and(|length| usize::try_from(length).map_or(true, |size| !is_body_size_ok(size)))
    {
        return Some("body_too_large");
    }
    None
}

fn log_refresh_rejection(
    reason: &'static str,
    status: Option<u16>,
    entries_observed: Option<usize>,
) {
    log::warn!(
        "event=litellm_refresh_rejected reason={reason} status={} entries_observed={}",
        status.map_or_else(|| "unknown".to_string(), |value| value.to_string()),
        entries_observed.map_or_else(|| "unknown".to_string(), |value| value.to_string())
    );
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct CatalogRefreshRejection {
    reason: &'static str,
    entries_observed: usize,
}

impl CatalogRefreshRejection {
    pub(super) const fn reason(self) -> &'static str {
        self.reason
    }

    pub(super) const fn entries_observed(self) -> usize {
        self.entries_observed
    }
}

pub(super) async fn publish_catalog(
    cache: &std::path::Path,
    registry: &tokio::sync::RwLock<
        std::collections::HashMap<String, super::litellm_catalog::ModelEntry>,
    >,
    body: &str,
) -> Result<(), CatalogRefreshRejection> {
    let catalog = super::litellm_catalog_parser::parse_catalog_detailed(body).map_err(
        |(reason, entries): (CatalogParseError, usize)| CatalogRefreshRejection {
            reason: reason.as_str(),
            entries_observed: entries,
        },
    )?;
    if catalog.len() < 100 {
        return Err(CatalogRefreshRejection {
            reason: "too_few_entries",
            entries_observed: catalog.len(),
        });
    }
    if let Some(parent) = cache.parent() {
        if std::fs::create_dir_all(parent).is_err() {
            return Err(CatalogRefreshRejection {
                reason: "cache_directory_unavailable",
                entries_observed: catalog.len(),
            });
        }
    }
    if crate::services::private_store::atomic_write(cache, body.as_bytes()).is_err() {
        return Err(CatalogRefreshRejection {
            reason: "cache_write_failed",
            entries_observed: catalog.len(),
        });
    }
    *registry.write().await = catalog;
    Ok(())
}

async fn read_body(response: reqwest::Response) -> Result<String, &'static str> {
    let mut bytes = Vec::with_capacity(
        response
            .content_length()
            .and_then(|length| usize::try_from(length).ok())
            .unwrap_or_default()
            .min(MAX_BODY_BYTES),
    );
    let mut stream = response.bytes_stream();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|_| "body_stream_failed")?;
        if bytes
            .len()
            .checked_add(chunk.len())
            .is_none_or(|length| length > MAX_BODY_BYTES)
        {
            return Err("body_too_large");
        }
        bytes.extend_from_slice(&chunk);
    }
    String::from_utf8(bytes).map_err(|_| "body_invalid_utf8")
}
