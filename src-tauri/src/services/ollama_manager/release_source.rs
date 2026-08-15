#![allow(dead_code)]

use super::error::OllamaErrorCode;
use super::fingerprint::{OllamaVersion, Sha256Digest};
use std::fmt;
use url::Url;

const MAX_ARCHIVES: usize = 2;
const MAX_ARCHIVE_NAME_BYTES: usize = 96;
const MAX_MANIFEST_BYTES: usize = 8 * 1024;
const ALLOWED_HOST: &str = "github.com";

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct AllowlistedArchiveName(String);

impl AllowlistedArchiveName {
    pub fn parse(raw: &str) -> Result<Self, OllamaErrorCode> {
        if raw.is_empty()
            || raw.len() > MAX_ARCHIVE_NAME_BYTES
            || !raw.is_ascii()
            || raw.contains(['/', '\\', '\0'])
            || raw == "."
            || raw == ".."
        {
            return Err(OllamaErrorCode::OllamaBundleInvalid);
        }
        let allowed = [
            "ollama-darwin.tgz",
            "ollama-linux-amd64.tar.zst",
            "ollama-linux-amd64-rocm.tar.zst",
            "ollama-windows-amd64.zip",
        ];
        allowed
            .contains(&raw)
            .then(|| Self(raw.to_owned()))
            .ok_or(OllamaErrorCode::OllamaBundleInvalid)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidatedHttpsUrl(Url);

impl ValidatedHttpsUrl {
    pub fn parse(raw: &str) -> Result<Self, OllamaErrorCode> {
        let parsed = Url::parse(raw).map_err(|_| OllamaErrorCode::OllamaBundleInvalid)?;
        if parsed.scheme() != "https"
            || parsed.host_str() != Some(ALLOWED_HOST)
            || parsed.port().is_some()
            || !parsed.username().is_empty()
            || parsed.password().is_some()
            || parsed.query().is_some()
            || parsed.fragment().is_some()
        {
            return Err(OllamaErrorCode::OllamaBundleInvalid);
        }
        Ok(Self(parsed))
    }

    pub fn as_url(&self) -> &Url {
        &self.0
    }
}

pub(crate) fn is_allowlisted_redirect(url: &Url) -> bool {
    ValidatedHttpsUrl::parse(url.as_str()).is_ok()
}

pub(crate) fn allowlisted_redirect_policy() -> reqwest::redirect::Policy {
    reqwest::redirect::Policy::custom(|attempt| {
        if attempt.previous().len() >= 3 || !is_allowlisted_redirect(attempt.url()) {
            attempt.stop()
        } else {
            attempt.follow()
        }
    })
}

impl fmt::Display for ValidatedHttpsUrl {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OllamaArchive {
    pub file_name: AllowlistedArchiveName,
    pub url: ValidatedHttpsUrl,
    pub expected_size: u64,
    pub sha256: Sha256Digest,
}

impl OllamaArchive {
    pub fn new(
        file_name: &str,
        url: ValidatedHttpsUrl,
        expected_size: u64,
        sha256: &str,
    ) -> Result<Self, OllamaErrorCode> {
        let file_name = AllowlistedArchiveName::parse(file_name)?;
        if expected_size == 0 {
            return Err(OllamaErrorCode::OllamaBundleInvalid);
        }
        let sha256 =
            Sha256Digest::from_hex(sha256).map_err(|_| OllamaErrorCode::OllamaBundleInvalid)?;
        Ok(Self {
            file_name,
            url,
            expected_size,
            sha256,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OllamaReleaseManifest {
    pub version: OllamaVersion,
    archives: Vec<OllamaArchive>,
}

impl OllamaReleaseManifest {
    pub fn try_new(
        version: OllamaVersion,
        archives: Vec<OllamaArchive>,
    ) -> Result<Self, OllamaErrorCode> {
        if archives.is_empty() || archives.len() > MAX_ARCHIVES {
            return Err(OllamaErrorCode::OllamaBundleInvalid);
        }
        for (index, archive) in archives.iter().enumerate() {
            if archives[..index]
                .iter()
                .any(|previous| previous.file_name == archive.file_name)
            {
                return Err(OllamaErrorCode::OllamaBundleInvalid);
            }
            validate_release_url(archive, &version)?;
        }
        Ok(Self { version, archives })
    }

    pub fn archives(&self) -> &[OllamaArchive] {
        &self.archives
    }
}

fn validate_release_url(
    archive: &OllamaArchive,
    version: &OllamaVersion,
) -> Result<(), OllamaErrorCode> {
    let expected = format!(
        "/ollama/ollama/releases/download/v{}/{}",
        version,
        archive.file_name.as_str()
    );
    (archive.url.as_url().path() == expected)
        .then_some(())
        .ok_or(OllamaErrorCode::OllamaBundleInvalid)
}

pub fn normalize_remote_version(raw: &str) -> Result<OllamaVersion, OllamaErrorCode> {
    let normalized = raw.trim().strip_prefix('v').unwrap_or(raw.trim());
    OllamaVersion::parse(normalized).map_err(|_| OllamaErrorCode::OllamaBundleInvalid)
}

pub fn fallback_version() -> Result<OllamaVersion, OllamaErrorCode> {
    normalize_remote_version(include_str!("../../../ollama-version.txt"))
}

pub fn parse_sha256_manifest(
    content: &[u8],
    archive_name: &AllowlistedArchiveName,
) -> Result<Sha256Digest, OllamaErrorCode> {
    if content.len() > MAX_MANIFEST_BYTES {
        return Err(OllamaErrorCode::OllamaChecksumMismatch);
    }
    let text = std::str::from_utf8(content).map_err(|_| OllamaErrorCode::OllamaChecksumMismatch)?;
    for line in text.lines().map(str::trim).filter(|line| !line.is_empty()) {
        let Some((hash, name)) = line.split_once(char::is_whitespace) else {
            continue;
        };
        let name = name.trim().trim_start_matches("./");
        if name != archive_name.as_str() {
            continue;
        }
        return Sha256Digest::from_hex(hash.trim())
            .map_err(|_| OllamaErrorCode::OllamaChecksumMismatch);
    }
    Err(OllamaErrorCode::OllamaChecksumMismatch)
}

pub(crate) use super::release_fetch::fetch_manifest;
