use std::time::{Duration, Instant};

use serde::Deserialize;
use tokio_util::sync::CancellationToken;
use zeroize::{Zeroize, Zeroizing};

use super::{oauth_http, DeviceAuthorization, LlmOAuthProvider, OAuthFailure, TokenBundle};
use crate::services::secure_http::{read_json_bounded, OAUTH_BODY_LIMIT};

const MAX_WAIT: Duration = Duration::from_secs(900);
const MAX_SERVER_EXPIRY_SECONDS: u64 = 3_600;

pub struct DeviceFlowConfig {
    pub provider: LlmOAuthProvider,
    pub client_id: &'static str,
    pub device_url: &'static str,
    pub token_url: &'static str,
    pub scope: Option<&'static str>,
}

#[derive(Deserialize)]
struct DeviceWire {
    device_code: String,
    user_code: String,
    #[serde(default)]
    verification_uri: String,
    verification_uri_complete: Option<String>,
    expires_in: Option<u64>,
    interval: Option<u64>,
}

impl Drop for DeviceWire {
    fn drop(&mut self) {
        self.device_code.zeroize();
        self.user_code.zeroize();
    }
}

#[derive(Deserialize)]
struct PollError {
    error: Option<String>,
}

pub async fn request(config: &DeviceFlowConfig) -> Result<DeviceAuthorization, OAuthFailure> {
    let mut form = vec![("client_id", config.client_id)];
    if let Some(scope) = config.scope {
        form.push(("scope", scope));
    }
    let response = oauth_http::post_form(config.provider, config.device_url, &form).await?;
    if !response.status().is_success() {
        return Err(oauth_http::parse_error(response).await);
    }
    let mut wire: DeviceWire = read_json_bounded(response, OAUTH_BODY_LIMIT)
        .await
        .map_err(|_| OAuthFailure::Generic)?;
    validate_wire(config.provider, &wire)?;
    Ok(DeviceAuthorization {
        device_code: Zeroizing::new(std::mem::take(&mut wire.device_code)),
        user_code: std::mem::take(&mut wire.user_code),
        verification_uri: std::mem::take(&mut wire.verification_uri),
        verification_uri_complete: std::mem::take(&mut wire.verification_uri_complete),
        interval_seconds: wire.interval.unwrap_or(5).clamp(1, 30),
        expires_in_seconds: wire
            .expires_in
            .unwrap_or(MAX_WAIT.as_secs())
            .min(MAX_WAIT.as_secs()),
    })
}

pub async fn poll(
    config: &DeviceFlowConfig,
    authorization: &DeviceAuthorization,
    cancel: &CancellationToken,
) -> Result<TokenBundle, OAuthFailure> {
    let started = Instant::now();
    let mut interval = Duration::from_secs(authorization.interval_seconds);
    let expires = Duration::from_secs(authorization.expires_in_seconds).min(MAX_WAIT);
    loop {
        if cancel.is_cancelled() {
            return Err(OAuthFailure::Cancelled);
        }
        if started.elapsed() >= expires {
            return Err(OAuthFailure::Expired);
        }
        tokio::select! {
            _ = cancel.cancelled() => return Err(OAuthFailure::Cancelled),
            _ = tokio::time::sleep(interval) => {}
        }
        if started.elapsed() >= expires {
            return Err(OAuthFailure::Expired);
        }
        let response = oauth_http::post_form(
            config.provider,
            config.token_url,
            &[
                ("grant_type", "urn:ietf:params:oauth:grant-type:device_code"),
                ("device_code", authorization.device_code.as_str()),
                ("client_id", config.client_id),
            ],
        )
        .await?;
        if response.status().is_success() {
            return oauth_http::parse_token(response, None).await;
        }
        let error: PollError = read_json_bounded(response, OAUTH_BODY_LIMIT)
            .await
            .map_err(|_| OAuthFailure::Generic)?;
        match poll_error_action(error.error.as_deref())? {
            PollAction::Continue => {}
            PollAction::SlowDown => {
                interval = (interval + Duration::from_secs(5)).min(Duration::from_secs(30))
            }
        }
    }
}

pub fn public_verification_url(
    authorization: &DeviceAuthorization,
) -> Result<String, OAuthFailure> {
    if !authorization.verification_uri.is_empty() {
        return Ok(authorization.verification_uri.clone());
    }
    let complete = authorization
        .verification_uri_complete
        .as_deref()
        .ok_or(OAuthFailure::Generic)?;
    let mut url = url::Url::parse(complete).map_err(|_| OAuthFailure::Generic)?;
    url.set_query(None);
    url.set_fragment(None);
    Ok(url.to_string())
}

fn validate_wire(provider: LlmOAuthProvider, wire: &DeviceWire) -> Result<(), OAuthFailure> {
    if wire.device_code.is_empty()
        || wire.device_code.len() > 4_096
        || wire.user_code.is_empty()
        || wire.user_code.len() > 64
        || wire
            .expires_in
            .is_some_and(|seconds| !(60..=MAX_SERVER_EXPIRY_SECONDS).contains(&seconds))
        || (wire.verification_uri.is_empty() && wire.verification_uri_complete.is_none())
        || (!wire.verification_uri.is_empty()
            && !trusted_verification_url(provider, &wire.verification_uri))
        || wire
            .verification_uri_complete
            .as_deref()
            .is_some_and(|url| !trusted_verification_url(provider, url))
    {
        return Err(OAuthFailure::Generic);
    }
    Ok(())
}

#[derive(Debug, PartialEq, Eq)]
enum PollAction {
    Continue,
    SlowDown,
}

fn poll_error_action(error: Option<&str>) -> Result<PollAction, OAuthFailure> {
    match error {
        Some("authorization_pending") => Ok(PollAction::Continue),
        Some("slow_down") => Ok(PollAction::SlowDown),
        Some("access_denied" | "authorization_denied") => Err(OAuthFailure::Denied),
        Some("expired_token") => Err(OAuthFailure::Expired),
        _ => Err(OAuthFailure::Generic),
    }
}

fn trusted_verification_url(provider: LlmOAuthProvider, value: &str) -> bool {
    if value.len() > 512 {
        return false;
    }
    let Ok(url) = url::Url::parse(value) else {
        return false;
    };
    let Some(host) = url.host_str() else {
        return false;
    };
    let trusted = match provider {
        LlmOAuthProvider::Xai => host == "x.ai" || host.ends_with(".x.ai"),
        LlmOAuthProvider::Kimi => host == "kimi.com" || host.ends_with(".kimi.com"),
    };
    url.scheme() == "https" && trusted && url.username().is_empty() && url.password().is_none()
}

#[cfg(test)]
#[path = "device_flow_tests.rs"]
mod tests;
