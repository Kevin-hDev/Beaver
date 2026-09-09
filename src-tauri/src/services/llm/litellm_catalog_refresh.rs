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
        Err(_) => return,
    };
    let cached = cache_path();
    let mut request = client.get(SOURCE_URL);
    if let Ok(modified) = std::fs::metadata(&cached).and_then(|metadata| metadata.modified()) {
        request = request.header("If-Modified-Since", httpdate::fmt_http_date(modified));
    }
    let response = match request.send().await {
        Ok(response) => response,
        Err(_) => return,
    };
    if response.status() == 304 || !response.status().is_success() {
        return;
    }
    if !response.url().host_str().is_some_and(is_trusted_host) {
        return;
    }
    if !is_body_size_ok(response.content_length().unwrap_or(0) as usize) {
        return;
    }
    let body = match read_body(response).await {
        Some(body) => body,
        None => return,
    };
    if let Err(rejection) = publish_catalog(&cached, get_lock(), &body).await {
        log::warn!(
            "event=litellm_refresh_rejected reason={} entries={}",
            rejection.reason(),
            rejection.entries()
        );
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct CatalogRefreshRejection {
    reason: &'static str,
    entries: usize,
}

impl CatalogRefreshRejection {
    pub(super) const fn reason(self) -> &'static str {
        self.reason
    }

    pub(super) const fn entries(self) -> usize {
        self.entries
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
            entries,
        },
    )?;
    if catalog.len() < 100 {
        return Err(CatalogRefreshRejection {
            reason: "too_few_entries",
            entries: catalog.len(),
        });
    }
    if let Some(parent) = cache.parent() {
        if std::fs::create_dir_all(parent).is_err() {
            return Err(CatalogRefreshRejection {
                reason: "cache_directory_unavailable",
                entries: catalog.len(),
            });
        }
    }
    if crate::services::private_store::atomic_write(cache, body.as_bytes()).is_err() {
        return Err(CatalogRefreshRejection {
            reason: "cache_write_failed",
            entries: catalog.len(),
        });
    }
    *registry.write().await = catalog;
    Ok(())
}

async fn read_body(response: reqwest::Response) -> Option<String> {
    let mut bytes = Vec::with_capacity(
        response
            .content_length()
            .and_then(|length| usize::try_from(length).ok())
            .unwrap_or_default()
            .min(MAX_BODY_BYTES),
    );
    let mut stream = response.bytes_stream();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.ok()?;
        if bytes.len().checked_add(chunk.len())? > MAX_BODY_BYTES {
            return None;
        }
        bytes.extend_from_slice(&chunk);
    }
    String::from_utf8(bytes).ok()
}
