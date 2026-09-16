use std::path::Path;

use reqwest::{header, Response, StatusCode, Url};

use super::{save_checkpoint, Checkpoint, HttpValidator};

const ALLOWED_DOWNLOAD_HOSTS: [&str; 4] = [
    "github.com",
    "objects.githubusercontent.com",
    "release-assets.githubusercontent.com",
    "github-releases.githubusercontent.com",
];

pub(super) async fn send(
    client: &reqwest::Client,
    initial_url: &str,
    checkpoint: &Checkpoint,
    allow_loopback: bool,
    max_redirects: u8,
) -> Result<Response, String> {
    let mut url = Url::parse(initial_url).map_err(|_| network_error())?;
    for redirect in 0..=max_redirects {
        if !url_allowed(&url, allow_loopback) {
            return Err(network_error());
        }
        let mut request = client.get(url.clone());
        if checkpoint.durable_bytes > 0 {
            request = request.header(
                header::RANGE,
                format!("bytes={}-", checkpoint.durable_bytes),
            );
            if let Some(value) = validator_value(checkpoint.validator.as_ref()) {
                request = request.header(header::IF_RANGE, value);
            }
        }
        let response = request.send().await.map_err(|_| network_error())?;
        if !response.status().is_redirection() {
            return Ok(response);
        }
        if redirect == max_redirects {
            return Err(network_error());
        }
        let location = response
            .headers()
            .get(header::LOCATION)
            .and_then(|value| value.to_str().ok())
            .ok_or_else(network_error)?;
        url = url.join(location).map_err(|_| network_error())?;
    }
    Err(network_error())
}

pub(super) fn normalize_response(
    response: Response,
    checkpoint: &mut Checkpoint,
    partial_file: &Path,
    checkpoint_file: &Path,
) -> Result<Response, String> {
    if checkpoint.durable_bytes == 0 {
        let accepted = response.status() == StatusCode::OK
            || (response.status() == StatusCode::PARTIAL_CONTENT
                && valid_content_range(response.headers(), 0));
        return accepted.then_some(response).ok_or_else(network_error);
    }
    if response.status() == StatusCode::OK {
        reset_partial(partial_file, checkpoint_file, checkpoint)?;
        return Ok(response);
    }
    if response.status() != StatusCode::PARTIAL_CONTENT
        || !valid_content_range(response.headers(), checkpoint.durable_bytes)
        || validator_changed(checkpoint.validator.as_ref(), response.headers())
    {
        reset_partial(partial_file, checkpoint_file, checkpoint)?;
        return Err(network_error());
    }
    Ok(response)
}

pub(super) fn response_validator(headers: &header::HeaderMap) -> Option<HttpValidator> {
    headers
        .get(header::ETAG)
        .and_then(|value| value.to_str().ok())
        .map(|value| HttpValidator::Etag(value.to_string()))
        .or_else(|| {
            headers
                .get(header::LAST_MODIFIED)
                .and_then(|value| value.to_str().ok())
                .map(|value| HttpValidator::LastModified(value.to_string()))
        })
        .filter(|validator| {
            let value = validator_value(Some(validator)).unwrap_or_default();
            !value.is_empty()
                && value.chars().count() <= 512
                && !value.chars().any(char::is_control)
        })
}

fn reset_partial(
    partial_file: &Path,
    checkpoint_file: &Path,
    checkpoint: &mut Checkpoint,
) -> Result<(), String> {
    ::log::warn!(
        "[voice-download] model={} step=partial-reset",
        checkpoint.entry_id
    );
    std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(partial_file)
        .and_then(|file| file.sync_all())
        .map_err(|_| "model-download-storage-failed".to_string())?;
    checkpoint.durable_bytes = 0;
    checkpoint.validator = None;
    save_checkpoint(checkpoint_file, checkpoint)
}

fn valid_content_range(headers: &header::HeaderMap, start: u64) -> bool {
    headers
        .get(header::CONTENT_RANGE)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| value.starts_with(&format!("bytes {start}-")))
}

fn validator_changed(expected: Option<&HttpValidator>, headers: &header::HeaderMap) -> bool {
    expected.is_some_and(|expected| response_validator(headers).as_ref() != Some(expected))
}

fn validator_value(validator: Option<&HttpValidator>) -> Option<&str> {
    match validator? {
        HttpValidator::Etag(value) | HttpValidator::LastModified(value) => Some(value),
    }
}

pub(super) fn production_url_allowed(value: &str) -> bool {
    value.chars().count() <= crate::services::voice::limits::MAX_CATALOG_URL_CHARS
        && Url::parse(value).is_ok_and(|url| url_allowed(&url, false))
}

fn url_allowed(url: &Url, allow_loopback: bool) -> bool {
    if !url.username().is_empty() || url.password().is_some() {
        return false;
    }
    if url.scheme() == "https"
        && url
            .host_str()
            .is_some_and(|host| ALLOWED_DOWNLOAD_HOSTS.contains(&host))
    {
        return true;
    }
    allow_loopback
        && url.scheme() == "http"
        && matches!(url.host(), Some(url::Host::Ipv4(ip)) if ip.is_loopback())
}

fn network_error() -> String {
    "model-download-network-failed".into()
}
