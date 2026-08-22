use std::time::Duration;

use serde::Deserialize;
use subtle::ConstantTimeEq;
use zeroize::{Zeroize, Zeroizing};

use super::{headers, LlmOAuthProvider, OAuthFailure, TokenBundle};
use crate::services::secure_http::{read_json_bounded, AuthenticatedClient, OAUTH_BODY_LIMIT};

const USER_TIMEOUT: Duration = Duration::from_secs(10);
const MAX_USER_ID_BYTES: usize = 256;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct UserWire {
    user_id: String,
}

impl Drop for UserWire {
    fn drop(&mut self) {
        self.user_id.zeroize();
    }
}

pub async fn enrich(
    tokens: &mut TokenBundle,
    previous_user_id: Option<&str>,
) -> Result<(), OAuthFailure> {
    let user_id = fetch(&tokens.access).await?;
    if previous_user_id.is_some_and(|previous| !same_identity(previous, &user_id)) {
        return Err(OAuthFailure::Unauthorized);
    }
    tokens.user_id = Some(user_id);
    Ok(())
}

async fn fetch(token: &str) -> Result<Zeroizing<String>, OAuthFailure> {
    let client = AuthenticatedClient::new(USER_TIMEOUT).map_err(|_| OAuthFailure::Generic)?;
    let headers =
        headers::request_headers(LlmOAuthProvider::Xai).map_err(|_| OAuthFailure::Generic)?;
    let response = client
        .send(client.get(user_url()).headers(headers).bearer_auth(token))
        .await
        .map_err(|_| OAuthFailure::Generic)?;
    if matches!(response.status().as_u16(), 401 | 403) {
        return Err(OAuthFailure::Unauthorized);
    }
    if !response.status().is_success() {
        return Err(OAuthFailure::Generic);
    }
    let mut wire: UserWire = read_json_bounded(response, OAUTH_BODY_LIMIT)
        .await
        .map_err(|_| OAuthFailure::Generic)?;
    validate_user_id(&wire.user_id)?;
    Ok(Zeroizing::new(std::mem::take(&mut wire.user_id)))
}

fn user_url() -> String {
    format!("{}/user", super::xai_headers::PROXY_BASE_URL)
}

fn validate_user_id(value: &str) -> Result<(), OAuthFailure> {
    if value.is_empty()
        || value.len() > MAX_USER_ID_BYTES
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
    {
        return Err(OAuthFailure::Generic);
    }
    Ok(())
}

fn same_identity(left: &str, right: &str) -> bool {
    left.len() == right.len() && left.as_bytes().ct_eq(right.as_bytes()).into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identity_is_bounded_and_compared() {
        assert!(validate_user_id("user_fixture-1").is_ok());
        assert!(validate_user_id("").is_err());
        assert!(validate_user_id("bad/user").is_err());
        assert!(validate_user_id(&"a".repeat(MAX_USER_ID_BYTES + 1)).is_err());
        assert!(same_identity("principal-a", "principal-a"));
        assert!(!same_identity("principal-a", "principal-b"));
    }

    #[test]
    fn user_endpoint_stays_on_the_subscription_proxy() {
        assert_eq!(user_url(), "https://cli-chat-proxy.grok.com/v1/user");
        assert!(!user_url().contains("api.x.ai"));
    }
}
