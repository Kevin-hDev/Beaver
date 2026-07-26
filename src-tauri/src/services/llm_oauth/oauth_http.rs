use std::time::Duration;

use reqwest::Response;
use serde::Deserialize;
use zeroize::{Zeroize, Zeroizing};

use super::{headers, LlmOAuthProvider, OAuthFailure, TokenBundle};
use crate::services::secure_http::{read_json_bounded, AuthenticatedClient, OAUTH_BODY_LIMIT};

const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);

#[derive(Deserialize)]
struct TokenWire {
    access_token: String,
    refresh_token: Option<String>,
    expires_in: Option<i64>,
}

impl Drop for TokenWire {
    fn drop(&mut self) {
        self.access_token.zeroize();
        if let Some(value) = &mut self.refresh_token {
            value.zeroize();
        }
    }
}

#[derive(Deserialize)]
struct ErrorWire {
    error: Option<String>,
}

impl Drop for ErrorWire {
    fn drop(&mut self) {
        if let Some(value) = &mut self.error {
            value.zeroize();
        }
    }
}

pub async fn post_form(
    provider: LlmOAuthProvider,
    url: &str,
    form: &[(&str, &str)],
) -> Result<Response, OAuthFailure> {
    let client = AuthenticatedClient::new(REQUEST_TIMEOUT).map_err(|_| OAuthFailure::Generic)?;
    let headers = headers::request_headers(provider).map_err(|_| OAuthFailure::Generic)?;
    client
        .send(client.post(url).headers(headers).form(form))
        .await
        .map_err(|_| OAuthFailure::Generic)
}

pub async fn parse_token(
    response: Response,
    fallback_refresh: Option<&str>,
) -> Result<TokenBundle, OAuthFailure> {
    if !response.status().is_success() {
        return Err(parse_error(response).await);
    }
    let wire: TokenWire = read_json_bounded(response, OAUTH_BODY_LIMIT)
        .await
        .map_err(|_| OAuthFailure::Generic)?;
    token_bundle(wire, fallback_refresh)
}

fn token_bundle(
    mut wire: TokenWire,
    fallback_refresh: Option<&str>,
) -> Result<TokenBundle, OAuthFailure> {
    let expires_in = wire.expires_in.ok_or(OAuthFailure::Generic)?;
    if !(60..=31_536_000).contains(&expires_in) || wire.access_token.is_empty() {
        return Err(OAuthFailure::Generic);
    }
    let refresh = wire
        .refresh_token
        .take()
        .or_else(|| fallback_refresh.map(ToOwned::to_owned))
        .filter(|value| !value.is_empty())
        .ok_or(OAuthFailure::Generic)?;
    Ok(TokenBundle {
        access: Zeroizing::new(std::mem::take(&mut wire.access_token)),
        refresh: Zeroizing::new(refresh),
        expires_at: chrono::Utc::now().timestamp().saturating_add(expires_in),
    })
}

pub async fn refresh(
    provider: LlmOAuthProvider,
    token_url: &str,
    client_id: &str,
    refresh_token: &str,
) -> Result<TokenBundle, OAuthFailure> {
    let response = post_form(
        provider,
        token_url,
        &[
            ("grant_type", "refresh_token"),
            ("refresh_token", refresh_token),
            ("client_id", client_id),
        ],
    )
    .await?;
    parse_token(response, Some(refresh_token)).await
}

pub async fn parse_error(response: Response) -> OAuthFailure {
    if matches!(response.status().as_u16(), 401 | 403) {
        return OAuthFailure::Unauthorized;
    }
    let error: Result<ErrorWire, _> = read_json_bounded(response, OAUTH_BODY_LIMIT).await;
    let Some(error) = error.ok() else {
        return OAuthFailure::Generic;
    };
    match error.error.as_deref() {
        Some("authorization_pending") | Some("slow_down") => OAuthFailure::Generic,
        Some("access_denied") => OAuthFailure::Denied,
        Some("expired_token") => OAuthFailure::Expired,
        Some("invalid_grant") => OAuthFailure::Unauthorized,
        _ => OAuthFailure::Generic,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use subtle::ConstantTimeEq;

    #[test]
    fn rejects_missing_expiry_and_keeps_rotated_refresh() {
        let missing = TokenWire {
            access_token: uuid::Uuid::new_v4().to_string(),
            refresh_token: Some(uuid::Uuid::new_v4().to_string()),
            expires_in: None,
        };
        assert!(matches!(
            token_bundle(missing, None),
            Err(OAuthFailure::Generic)
        ));

        let expected = Zeroizing::new(uuid::Uuid::new_v4().to_string());
        let rotated = TokenWire {
            access_token: uuid::Uuid::new_v4().to_string(),
            refresh_token: Some(expected.to_string()),
            expires_in: Some(3_600),
        };
        let fallback = Zeroizing::new(uuid::Uuid::new_v4().to_string());
        let tokens = token_bundle(rotated, Some(&fallback)).unwrap();
        assert!(bool::from(
            tokens.refresh.as_bytes().ct_eq(expected.as_bytes())
        ));
    }
}
