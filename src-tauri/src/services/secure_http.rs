use std::fmt;
use std::time::Duration;

use futures_util::StreamExt;
use reqwest::redirect::Policy;
use reqwest::{IntoUrl, RequestBuilder, Response};
use serde::de::DeserializeOwned;
use zeroize::Zeroizing;

pub const OAUTH_BODY_LIMIT: usize = 512 * 1024;
pub const TOKEN_BODY_LIMIT: usize = 512 * 1024;
pub const TELEGRAM_BODY_LIMIT: usize = 2 * 1024 * 1024;
pub const SLACK_BODY_LIMIT: usize = 512 * 1024;
pub const DISCORD_BODY_LIMIT: usize = 512 * 1024;
pub const MCP_BODY_LIMIT: usize = 10 * 1024 * 1024;
pub const LLM_BODY_LIMIT: usize = 10 * 1024 * 1024;
pub const PROVIDER_ERROR_LIMIT: usize = 16 * 1024;
pub const MAX_AUTHENTICATED_BODY_LIMIT: usize = 32 * 1024 * 1024;
pub const MAX_AUTHENTICATED_TIMEOUT: Duration = Duration::from_secs(600);

#[derive(Clone, Copy)]
enum UrlPolicy {
    HttpsOnly,
    LoopbackHttp,
}

#[derive(Clone)]
pub struct AuthenticatedClient {
    client: reqwest::Client,
    url_policy: UrlPolicy,
}

impl AuthenticatedClient {
    pub fn new(timeout: Duration) -> Result<Self, SecureHttpError> {
        Self::build(timeout, UrlPolicy::HttpsOnly)
    }

    pub(crate) fn new_loopback(timeout: Duration) -> Result<Self, SecureHttpError> {
        Self::build(timeout, UrlPolicy::LoopbackHttp)
    }

    fn build(timeout: Duration, url_policy: UrlPolicy) -> Result<Self, SecureHttpError> {
        if timeout.is_zero() || timeout > MAX_AUTHENTICATED_TIMEOUT {
            return Err(SecureHttpError::Configuration);
        }
        let client = reqwest::Client::builder()
            .redirect(Policy::none())
            .connect_timeout(timeout.min(Duration::from_secs(10)))
            .timeout(timeout)
            .build()
            .map_err(|_| SecureHttpError::Configuration)?;
        Ok(Self { client, url_policy })
    }

    pub fn get<U: IntoUrl>(&self, url: U) -> RequestBuilder {
        self.client.get(url)
    }

    pub fn post<U: IntoUrl>(&self, url: U) -> RequestBuilder {
        self.client.post(url)
    }

    pub async fn send(&self, request: RequestBuilder) -> Result<Response, SecureHttpError> {
        let request = request.build().map_err(|_| SecureHttpError::Request)?;
        if !self.url_is_allowed(request.url()) {
            return Err(SecureHttpError::InsecureUrl);
        }
        let response = self
            .client
            .execute(request)
            .await
            .map_err(|_| SecureHttpError::Request)?;
        if response.status().is_redirection() {
            return Err(SecureHttpError::Redirect);
        }
        Ok(response)
    }

    pub async fn send_success(&self, request: RequestBuilder) -> Result<Response, SecureHttpError> {
        let response = self.send(request).await?;
        if response.status().is_success() {
            Ok(response)
        } else {
            Err(SecureHttpError::Status)
        }
    }

    fn url_is_allowed(&self, url: &reqwest::Url) -> bool {
        if url.scheme() == "https" {
            return true;
        }
        if url.scheme() != "http" || !matches!(self.url_policy, UrlPolicy::LoopbackHttp) {
            return false;
        }
        match url.host() {
            Some(url::Host::Ipv4(address)) => address.is_loopback(),
            Some(url::Host::Ipv6(address)) => address.is_loopback(),
            _ => false,
        }
    }
}

pub async fn read_bounded(
    response: Response,
    limit: usize,
) -> Result<Zeroizing<Vec<u8>>, SecureHttpError> {
    if limit == 0 || limit > MAX_AUTHENTICATED_BODY_LIMIT {
        return Err(SecureHttpError::Configuration);
    }
    if response
        .content_length()
        .is_some_and(|length| length > limit as u64)
    {
        return Err(SecureHttpError::BodyTooLarge);
    }
    let capacity = response
        .content_length()
        .and_then(|length| usize::try_from(length).ok())
        .unwrap_or(0)
        .min(limit);
    let mut body = Zeroizing::new(Vec::with_capacity(capacity));
    let mut stream = response.bytes_stream();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|_| SecureHttpError::Body)?;
        let next = body
            .len()
            .checked_add(chunk.len())
            .ok_or(SecureHttpError::BodyTooLarge)?;
        if next > limit {
            return Err(SecureHttpError::BodyTooLarge);
        }
        body.extend_from_slice(&chunk);
    }
    Ok(body)
}

pub async fn read_json_bounded<T: DeserializeOwned>(
    response: Response,
    limit: usize,
) -> Result<T, SecureHttpError> {
    let body = read_bounded(response, limit).await?;
    serde_json::from_slice(&body).map_err(|_| SecureHttpError::Body)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SecureHttpError {
    Configuration,
    InsecureUrl,
    Request,
    Redirect,
    Status,
    Body,
    BodyTooLarge,
}

impl fmt::Display for SecureHttpError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("requête réseau refusée")
    }
}

impl std::error::Error for SecureHttpError {}

#[cfg(test)]
#[path = "secure_http_tests.rs"]
mod tests;
