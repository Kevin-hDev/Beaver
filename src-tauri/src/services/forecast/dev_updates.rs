use futures_util::{stream, StreamExt};
use serde::Serialize;
use serde_json::Value;
use std::time::Duration;

use super::dev_update_sources::{self, SourceSpec};
use crate::services::{brand, forecast::sidecar_runtime, secure_http};

const RESPONSE_LIMIT: usize = 256 * 1024;
const MAX_CONCURRENT_CHECKS: usize = 4;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ForecastDevUpdate {
    pub id: String,
    pub display_name: String,
    pub kind: String,
    pub current: String,
    pub latest: String,
    pub source_url: String,
}

pub async fn check() -> Result<Vec<ForecastDevUpdate>, String> {
    if !enabled(cfg!(debug_assertions)) {
        return Ok(Vec::new());
    }
    let client = secure_http::AuthenticatedClient::new(Duration::from_secs(12))
        .map_err(|_| "forecast-dev-update-check-error".to_string())?;
    let sources =
        dev_update_sources::all().map_err(|_| "forecast-dev-update-check-error".to_string())?;
    let updates = stream::iter(sources)
        .map(|source| check_source(client.clone(), source))
        .buffer_unordered(MAX_CONCURRENT_CHECKS)
        .filter_map(|update| async move { update })
        .take(dev_update_sources::MAX_SOURCES)
        .collect::<Vec<_>>()
        .await;
    Ok(updates)
}

pub(crate) fn enabled(debug_build: bool) -> bool {
    debug_build
}

async fn check_source(
    client: secure_http::AuthenticatedClient,
    source: SourceSpec,
) -> Option<ForecastDevUpdate> {
    let request_url = request_url(source)?;
    let response = client
        .send_success(
            client
                .get(request_url)
                .header("User-Agent", format!("{} ForecastDev", brand::user_agent()))
                .header("Accept", "application/json"),
        )
        .await
        .ok()?;
    let body: Value = secure_http::read_json_bounded(response, RESPONSE_LIMIT)
        .await
        .ok()?;
    build_update(source, latest_value(source, &body)?)
}

fn request_url(source: SourceSpec) -> Option<String> {
    match source {
        SourceSpec::Pypi { package, .. } => Some(format!("https://pypi.org/pypi/{package}/json")),
        SourceSpec::Github { repository, .. } => Some(format!(
            "https://api.github.com/repos/{repository}/commits/main"
        )),
        SourceSpec::HuggingFace {
            repository,
            reference,
            ..
        } => Some(format!(
            "https://huggingface.co/api/models/{repository}/revision/{reference}"
        )),
    }
    .filter(|url| url.len() <= 512)
}

fn latest_value(source: SourceSpec, body: &Value) -> Option<&str> {
    match source {
        SourceSpec::Pypi { .. } => body.get("info")?.get("version")?.as_str(),
        SourceSpec::Github { .. } | SourceSpec::HuggingFace { .. } => body.get("sha")?.as_str(),
    }
}

fn build_update(source: SourceSpec, latest: &str) -> Option<ForecastDevUpdate> {
    let is_commit = matches!(
        source,
        SourceSpec::Github { .. } | SourceSpec::HuggingFace { .. }
    );
    let (id, name, kind, current, source_url) = source_details(source)?;
    if latest == current || !valid_remote_value(latest, is_commit) {
        return None;
    }
    Some(ForecastDevUpdate {
        id: id.to_string(),
        display_name: name.to_string(),
        kind: kind.to_string(),
        current,
        latest: latest.to_string(),
        source_url,
    })
}

fn source_details(
    source: SourceSpec,
) -> Option<(&'static str, &'static str, &'static str, String, String)> {
    match source {
        SourceSpec::Pypi {
            id,
            name,
            family,
            package,
        } => Some((
            id,
            name,
            "runtime",
            sidecar_runtime::locked_package_version(family, package).ok()?,
            format!("https://pypi.org/project/{package}/"),
        )),
        SourceSpec::Github {
            id,
            name,
            repository,
            current,
        } => Some((
            id,
            name,
            "runtime",
            current.to_string(),
            format!("https://github.com/{repository}"),
        )),
        SourceSpec::HuggingFace {
            id,
            name,
            repository,
            current,
            ..
        } => Some((
            id,
            name,
            "model",
            current.to_string(),
            format!("https://huggingface.co/{repository}"),
        )),
    }
}

fn valid_remote_value(value: &str, commit: bool) -> bool {
    if commit {
        return value.len() == 40 && value.bytes().all(|byte| byte.is_ascii_hexdigit());
    }
    !value.is_empty()
        && value.len() <= 64
        && value.bytes().all(|byte| {
            byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'!' | b'+' | b'-' | b'_')
        })
}

#[cfg(test)]
#[path = "dev_updates_tests.rs"]
mod tests;
