#![allow(dead_code)]

use super::release_source::{
    allowlisted_redirect_policy, parse_sha256_manifest, AllowlistedArchiveName, OllamaArchive,
    OllamaReleaseManifest, ValidatedHttpsUrl,
};
use super::{error::OllamaErrorCode, fingerprint::OllamaVersion};
use reqwest::header::LOCATION;

const OLLAMA_LATEST_RELEASE_URL: &str = "https://github.com/ollama/ollama/releases/latest";
const OLLAMA_RELEASE_TAG_PREFIX: &str = "/ollama/ollama/releases/tag/v";

pub(crate) async fn fetch_latest_version() -> Result<OllamaVersion, OllamaErrorCode> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .map_err(|_| OllamaErrorCode::OllamaDownloadFailed)?;
    let response = client
        .get(OLLAMA_LATEST_RELEASE_URL)
        .header("User-Agent", crate::services::brand::user_agent())
        .send()
        .await
        .map_err(|_| OllamaErrorCode::OllamaDownloadFailed)?;
    let status = response.status();
    let location = response
        .headers()
        .get(LOCATION)
        .and_then(|value| value.to_str().ok());
    let version = version_from_latest_redirect(status, location);
    if let Err(code) = version.as_ref() {
        log::warn!(
            "[ollama-update-check] stage=latest-release status={} code={}",
            status.as_u16(),
            code.as_str()
        );
    }
    version
}

pub(crate) fn version_from_latest_redirect(
    status: reqwest::StatusCode,
    location: Option<&str>,
) -> Result<OllamaVersion, OllamaErrorCode> {
    if !status.is_redirection() {
        return Err(OllamaErrorCode::OllamaDownloadFailed);
    }
    let location = location.ok_or(OllamaErrorCode::OllamaDownloadFailed)?;
    let url =
        ValidatedHttpsUrl::parse(location).map_err(|_| OllamaErrorCode::OllamaDownloadFailed)?;
    let tag = url
        .as_url()
        .path()
        .strip_prefix(OLLAMA_RELEASE_TAG_PREFIX)
        .filter(|tag| !tag.is_empty() && !tag.contains('/'))
        .ok_or(OllamaErrorCode::OllamaDownloadFailed)?;
    super::release_source::normalize_remote_version(tag)
        .map_err(|_| OllamaErrorCode::OllamaDownloadFailed)
}

pub(crate) async fn fetch_manifest(
    version: OllamaVersion,
    archive_names: &[&str],
) -> Result<OllamaReleaseManifest, OllamaErrorCode> {
    if archive_names.is_empty() || archive_names.len() > 2 {
        return Err(OllamaErrorCode::OllamaBundleInvalid);
    }
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .redirect(allowlisted_redirect_policy())
        .build()
        .map_err(|_| OllamaErrorCode::OllamaDownloadFailed)?;
    let checksum_url = release_url(&version, "sha256sum.txt")?;
    let checksum = client
        .get(checksum_url.as_url().clone())
        .send()
        .await
        .map_err(|_| OllamaErrorCode::OllamaChecksumMismatch)?
        .bytes()
        .await
        .map_err(|_| OllamaErrorCode::OllamaChecksumMismatch)?;
    let mut archives = Vec::with_capacity(2);
    for name in archive_names {
        let name = AllowlistedArchiveName::parse(name)?;
        let url = release_url(&version, name.as_str())?;
        let expected_size = client
            .head(url.as_url().clone())
            .send()
            .await
            .map_err(|_| OllamaErrorCode::OllamaDownloadFailed)?
            .content_length()
            .ok_or(OllamaErrorCode::OllamaDownloadFailed)?;
        let sha256 = parse_sha256_manifest(&checksum, &name)?.to_hex();
        archives.push(OllamaArchive::new(
            name.as_str(),
            url,
            expected_size,
            &sha256,
        )?);
    }
    OllamaReleaseManifest::try_new(version, archives)
}

fn release_url(version: &OllamaVersion, name: &str) -> Result<ValidatedHttpsUrl, OllamaErrorCode> {
    ValidatedHttpsUrl::parse(&format!(
        "https://github.com/ollama/ollama/releases/download/v{version}/{name}"
    ))
}
