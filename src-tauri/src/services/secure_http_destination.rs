use std::net::SocketAddr;
use std::time::Duration;

use reqwest::{RequestBuilder, Response, Url};

use super::secure_http::SecureHttpError;

pub struct FixedDestination {
    client: reqwest::Client,
    url: Url,
}

impl FixedDestination {
    pub fn new(url: Url, address: SocketAddr, timeout: Duration) -> Result<Self, SecureHttpError> {
        if url.scheme() != "https" || timeout.is_zero() {
            return Err(SecureHttpError::Configuration);
        }
        Self::build_pinned(url, address, timeout)
    }

    fn build_pinned(
        url: Url,
        address: SocketAddr,
        timeout: Duration,
    ) -> Result<Self, SecureHttpError> {
        let host = url.host_str().ok_or(SecureHttpError::Configuration)?;
        let client = reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .no_proxy()
            .resolve(host, address)
            .timeout(timeout)
            .build()
            .map_err(|_| SecureHttpError::Configuration)?;
        Ok(Self { client, url })
    }

    #[cfg(test)]
    pub fn new_test_pinned(
        url: Url,
        address: SocketAddr,
        timeout: Duration,
    ) -> Result<Self, SecureHttpError> {
        if url.scheme() != "http" || !address.ip().is_loopback() {
            return Err(SecureHttpError::Configuration);
        }
        Self::build_pinned(url, address, timeout)
    }

    #[cfg(test)]
    pub fn new_loopback(url: Url, timeout: Duration) -> Result<Self, SecureHttpError> {
        if url.scheme() != "http"
            || !url.host().is_some_and(|host| match host {
                url::Host::Ipv4(ip) => ip.is_loopback(),
                url::Host::Ipv6(ip) => ip.is_loopback(),
                url::Host::Domain(_) => false,
            })
        {
            return Err(SecureHttpError::Configuration);
        }
        let client = reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .no_proxy()
            .timeout(timeout)
            .build()
            .map_err(|_| SecureHttpError::Configuration)?;
        Ok(Self { client, url })
    }

    pub fn get(&self) -> RequestBuilder {
        self.client.get(self.url.clone())
    }

    pub fn post(&self) -> RequestBuilder {
        self.client.post(self.url.clone())
    }

    pub async fn send(&self, request: RequestBuilder) -> Result<Response, SecureHttpError> {
        let request = request.build().map_err(|_| SecureHttpError::Request)?;
        if request.url() != &self.url {
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
}

#[cfg(test)]
#[path = "secure_http_destination_tests.rs"]
mod tests;
