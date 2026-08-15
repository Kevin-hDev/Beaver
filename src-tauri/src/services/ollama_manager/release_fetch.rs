#![allow(dead_code)]

use super::release_source::{
    allowlisted_redirect_policy, parse_sha256_manifest, AllowlistedArchiveName, OllamaArchive,
    OllamaReleaseManifest, ValidatedHttpsUrl,
};
use super::{error::OllamaErrorCode, fingerprint::OllamaVersion};

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
