use super::release_source::ValidatedHttpsUrl;
use url::Url;

const ASSET_HOST: &str = "release-assets.githubusercontent.com";
const MAX_REDIRECT_URL_BYTES: usize = 2_048;

pub(crate) fn is_allowlisted_redirect(url: &Url) -> bool {
    ValidatedHttpsUrl::parse(url.as_str()).is_ok() || is_github_asset_url(url)
}

pub(crate) fn allowlisted_redirect_policy() -> reqwest::redirect::Policy {
    reqwest::redirect::Policy::custom(|attempt| {
        let previous = attempt.previous().last();
        if attempt.previous().len() >= 3
            || !previous.is_some_and(is_allowlisted_redirect)
            || !previous.is_some_and(|url| redirect_pair_is_allowed(url, attempt.url()))
        {
            attempt.stop()
        } else {
            attempt.follow()
        }
    })
}

pub(crate) fn redirect_pair_is_allowed(previous: &Url, next: &Url) -> bool {
    if ValidatedHttpsUrl::parse(previous.as_str()).is_ok() {
        return is_allowlisted_redirect(next);
    }
    is_github_asset_url(previous) && is_github_asset_url(next)
}

fn is_github_asset_url(url: &Url) -> bool {
    let path = url.path().to_ascii_lowercase();
    url.as_str().len() <= MAX_REDIRECT_URL_BYTES
        && url.scheme() == "https"
        && url.host_str() == Some(ASSET_HOST)
        && url.port().is_none()
        && url.username().is_empty()
        && url.password().is_none()
        && url.fragment().is_none()
        && url.query().is_some()
        && path.starts_with("/github-production-release-asset/")
        && !path.contains("..")
        && !path.contains("%2e")
}
