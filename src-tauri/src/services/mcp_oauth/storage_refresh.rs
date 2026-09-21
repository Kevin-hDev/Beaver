use zeroize::Zeroizing;

use super::storage::RefreshDependencies;
use super::types::{OAuthTokens, TokenResponse};

pub(super) async fn refresh_access_token(
    connector_id: &str,
    old: &OAuthTokens,
    refresh_token: &str,
    generation: u64,
    dependencies: &RefreshDependencies<'_>,
) -> Result<Zeroizing<String>, String> {
    (dependencies.validate)(connector_id, &old.token_endpoint)?;
    let client = (dependencies.client)(connector_id, &old.token_endpoint).await?;

    let mut params: Vec<(&str, &str)> = vec![
        ("grant_type", "refresh_token"),
        ("refresh_token", refresh_token),
        ("client_id", old.client_id.as_str()),
    ];
    let secret_ref = old
        .client_secret
        .as_ref()
        .map(|s| Zeroizing::new(s.as_str().to_string()));
    if let Some(ref secret) = secret_ref {
        params.push(("client_secret", secret.as_str()));
    }

    let request = client
        .post()
        .header("Accept", "application/json")
        .form(&params);
    let resp = client
        .send(request)
        .await
        .map_err(|_| "échec du rafraîchissement du token".to_string())?;
    if matches!(
        resp.status(),
        reqwest::StatusCode::UNAUTHORIZED | reqwest::StatusCode::FORBIDDEN
    ) {
        return Err(super::types::REAUTHENTICATION_REQUIRED.to_string());
    }
    if !resp.status().is_success() {
        return Err("échec du rafraîchissement du token".to_string());
    }

    let mut raw: TokenResponse = super::bounded_json(resp).await?;
    raw.validate()?;
    if raw.refresh_token.is_none() {
        raw.refresh_token = Some(refresh_token.to_string());
    }

    let cs = old.client_secret.as_ref().map(|s| s.as_str());
    let new_tokens = OAuthTokens::from_response(
        &mut raw,
        old.issuer
            .as_deref()
            .ok_or(super::types::REAUTHENTICATION_REQUIRED)?,
        &old.token_endpoint,
        &old.client_id,
        cs,
    );
    let result = new_tokens.access_token.clone();
    (dependencies.save)(connector_id, &new_tokens, generation)?;
    Ok(result)
}
