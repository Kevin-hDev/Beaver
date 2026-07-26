use std::time::Duration;

use reqwest::redirect::Policy;
use reqwest::{Client, RequestBuilder};
use url::Url;

use crate::services::{brand, secure_http::AuthenticatedClient};

#[path = "app_update_source_validation.rs"]
mod validation;

pub(crate) use validation::{is_safe_asset_name, is_safe_version, strict_version_gt};
use validation::{redirect_target_is_allowed, safe_repository_part, strict_url};

const API_HOST: &str = "api.github.com";
const ASSET_HOST: &str = "github.com";
const NOTES_HOST: &str = "raw.githubusercontent.com";
const REDIRECT_HOST: &str = "release-assets.githubusercontent.com";
const MAX_REDIRECTS: usize = 3;
pub(crate) const MAX_RELEASE_RESPONSE_BYTES: usize = 512 * 1024;
pub(crate) const UPDATE_MANIFEST_NAME: &str = "update-manifest.json";

pub(crate) struct UpdateSource {
    pub repository: &'static str,
    pub asset_prefix: &'static str,
    pub release_product: &'static str,
}

pub(crate) const UPDATE_SOURCE: UpdateSource = UpdateSource {
    repository: "Kevin-hDev/Beaver",
    asset_prefix: "Beaver_",
    release_product: "Beaver",
};

#[derive(Debug, Clone)]
pub(crate) struct AssetReference {
    pub version: String,
    pub name: String,
    pub url: Url,
}

impl UpdateSource {
    pub fn latest_release_url(&self) -> Option<Url> {
        let raw = format!(
            "https://{API_HOST}/repos/{}/releases/latest",
            self.repository_path()?
        );
        strict_url(&raw, API_HOST)
    }

    #[cfg(test)]
    pub fn is_latest_release_url(&self, raw: &str) -> bool {
        let Some(url) = strict_url(raw, API_HOST) else {
            return false;
        };
        url.path()
            == format!(
                "/repos/{}/releases/latest",
                self.repository_path().unwrap_or_default()
            )
    }

    pub fn release_notes_url(&self, version: &str) -> Option<Url> {
        if !is_safe_version(version) {
            return None;
        }
        let raw = format!(
            "https://{NOTES_HOST}/{}/v{version}/app-release-notes.json",
            self.repository_path()?
        );
        strict_url(&raw, NOTES_HOST)
    }

    pub fn manifest_url(&self, version: &str) -> Option<Url> {
        if !is_safe_version(version) {
            return None;
        }
        let raw = format!(
            "https://{ASSET_HOST}/{}/releases/download/v{version}/{UPDATE_MANIFEST_NAME}",
            self.repository_path()?
        );
        self.exact_manifest_url(&raw, version)
    }

    pub fn exact_manifest_url(&self, raw: &str, version: &str) -> Option<Url> {
        self.exact_release_file_url(raw, version, UPDATE_MANIFEST_NAME)
    }

    #[cfg(test)]
    pub fn is_manifest_url(&self, raw: &str, version: &str) -> bool {
        self.exact_manifest_url(raw, version).is_some()
    }

    #[cfg(test)]
    pub fn is_release_notes_url(&self, raw: &str, version: &str) -> bool {
        self.release_notes_url(version)
            .is_some_and(|expected| strict_url(raw, NOTES_HOST).as_ref() == Some(&expected))
    }

    pub fn asset_reference(&self, raw: &str) -> Option<AssetReference> {
        let reference = self.release_file_reference(raw)?;
        let version = &reference.version;
        let name = &reference.name;
        let expected_prefix = format!("{}{version}_", self.asset_prefix);
        if !name.starts_with(&expected_prefix) {
            return None;
        }
        Some(reference)
    }

    pub fn exact_asset_url(&self, raw: &str, version: &str, name: &str) -> Option<Url> {
        if !is_safe_version(version) || !is_safe_asset_name(name) {
            return None;
        }
        let reference = self.asset_reference(raw)?;
        (reference.version == version && reference.name == name).then_some(reference.url)
    }

    fn exact_release_file_url(&self, raw: &str, version: &str, name: &str) -> Option<Url> {
        if !is_safe_version(version) || !is_safe_asset_name(name) {
            return None;
        }
        let reference = self.release_file_reference(raw)?;
        (reference.version == version && reference.name == name).then_some(reference.url)
    }

    fn release_file_reference(&self, raw: &str) -> Option<AssetReference> {
        let url = strict_url(raw, ASSET_HOST)?;
        let prefix = format!("/{}/releases/download/v", self.repository_path()?);
        let rest = url.path().strip_prefix(&prefix)?;
        let (version, name) = rest.split_once('/')?;
        if name.contains('/') || !is_safe_version(version) || !is_safe_asset_name(name) {
            return None;
        }
        Some(AssetReference {
            version: version.to_string(),
            name: name.to_string(),
            url,
        })
    }

    fn repository_path(&self) -> Option<&str> {
        let (owner, repository) = self.repository.split_once('/')?;
        if self.repository.matches('/').count() != 1
            || !safe_repository_part(owner)
            || !safe_repository_part(repository)
        {
            return None;
        }
        Some(self.repository)
    }
}

pub(crate) fn update_request(request: RequestBuilder) -> RequestBuilder {
    request.header("User-Agent", brand::user_agent())
}

pub(crate) fn metadata_client() -> Result<AuthenticatedClient, ()> {
    AuthenticatedClient::new(Duration::from_secs(30)).map_err(|_| ())
}

pub(crate) fn download_client() -> Result<Client, ()> {
    Client::builder()
        .connect_timeout(Duration::from_secs(10))
        .timeout(Duration::from_secs(30 * 60))
        .redirect(Policy::custom(|attempt| {
            let previous = attempt.previous().last();
            if previous.is_some_and(|previous| {
                release_redirect_is_allowed(previous, attempt.url(), attempt.previous().len())
            }) {
                attempt.follow()
            } else {
                attempt.error(std::io::Error::other("redirection refusée"))
            }
        }))
        .build()
        .map_err(|_| ())
}

pub(crate) fn release_redirect_is_allowed(
    previous: &Url,
    next: &Url,
    redirects_so_far: usize,
) -> bool {
    let trusted_source = UPDATE_SOURCE.asset_reference(previous.as_str()).is_some()
        || UPDATE_SOURCE
            .release_file_reference(previous.as_str())
            .is_some_and(|reference| reference.name == UPDATE_MANIFEST_NAME);
    if redirects_so_far >= MAX_REDIRECTS || !trusted_source {
        return false;
    }
    redirect_target_is_allowed(next, REDIRECT_HOST)
}

#[cfg(test)]
#[path = "app_update_source_tests.rs"]
mod tests;
