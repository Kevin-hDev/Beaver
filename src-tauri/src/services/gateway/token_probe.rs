use std::time::Duration;

use serde::de::DeserializeOwned;
use serde::Deserialize;
use zeroize::Zeroizing;

use super::tokens::AccountTokens;
use crate::services::secure_http::{read_json_bounded, AuthenticatedClient, TOKEN_BODY_LIMIT};

const PROBE_TIMEOUT: Duration = Duration::from_secs(10);

pub(crate) struct ProbeEndpoints {
    telegram: String,
    slack: String,
    discord: String,
    allow_loopback_http: bool,
}

impl ProbeEndpoints {
    pub(crate) fn production() -> Self {
        Self {
            telegram: "https://api.telegram.org".into(),
            slack: "https://slack.com/api".into(),
            discord: "https://discord.com/api/v10".into(),
            allow_loopback_http: false,
        }
    }

    #[cfg(test)]
    fn all(base: String) -> Self {
        Self {
            telegram: base.clone(),
            slack: base.clone(),
            discord: base,
            allow_loopback_http: true,
        }
    }
}

pub(crate) async fn validate_tokens(
    channel_id: &str,
    credentials: &AccountTokens,
    endpoints: &ProbeEndpoints,
) -> Result<(), String> {
    credentials.validate_for(channel_id)?;
    let client = if endpoints.allow_loopback_http {
        AuthenticatedClient::new_loopback(PROBE_TIMEOUT)
    } else {
        AuthenticatedClient::new(PROBE_TIMEOUT)
    }
    .map_err(|_| generic_error())?;
    match channel_id {
        "telegram" => validate_telegram(&client, credentials, endpoints).await,
        "slack" => validate_slack(&client, credentials, endpoints).await,
        "discord" => validate_discord(&client, credentials, endpoints).await,
        _ => Err(generic_error()),
    }
}

async fn validate_telegram(
    client: &AuthenticatedClient,
    credentials: &AccountTokens,
    endpoints: &ProbeEndpoints,
) -> Result<(), String> {
    let token = credentials.token().ok_or_else(generic_error)?;
    let url = Zeroizing::new(format!("{}/bot{token}/getMe", endpoints.telegram));
    let body: TelegramResponse = checked_json(client, client.get(url.as_str())).await?;
    if body.ok && body.result.is_some_and(|user| !user.id.is_empty()) {
        Ok(())
    } else {
        Err(generic_error())
    }
}

async fn validate_slack(
    client: &AuthenticatedClient,
    credentials: &AccountTokens,
    endpoints: &ProbeEndpoints,
) -> Result<(), String> {
    let bot: SlackResponse = checked_json(
        client,
        client
            .post(format!("{}/auth.test", endpoints.slack))
            .bearer_auth(credentials.bot_token().ok_or_else(generic_error)?),
    )
    .await?;
    if !bot.ok || bot.user_id.is_none_or(|id| id.is_empty()) {
        return Err(generic_error());
    }
    let app: SlackResponse = checked_json(
        client,
        client
            .post(format!("{}/apps.connections.open", endpoints.slack))
            .bearer_auth(credentials.app_token().ok_or_else(generic_error)?),
    )
    .await?;
    if !app.ok || app.url.is_none_or(|url| !url.starts_with("wss://")) {
        return Err(generic_error());
    }
    Ok(())
}

async fn validate_discord(
    client: &AuthenticatedClient,
    credentials: &AccountTokens,
    endpoints: &ProbeEndpoints,
) -> Result<(), String> {
    let auth = Zeroizing::new(format!(
        "Bot {}",
        credentials.token().ok_or_else(generic_error)?
    ));
    let body: DiscordResponse = checked_json(
        client,
        client
            .get(format!("{}/users/@me", endpoints.discord))
            .header("Authorization", auth.as_str()),
    )
    .await?;
    if body.id.is_empty() {
        Err(generic_error())
    } else {
        Ok(())
    }
}

async fn checked_json<T: DeserializeOwned>(
    client: &AuthenticatedClient,
    request: reqwest::RequestBuilder,
) -> Result<T, String> {
    let response = client.send(request).await.map_err(|_| generic_error())?;
    if !response.status().is_success() {
        return Err(generic_error());
    }
    read_json_bounded(response, TOKEN_BODY_LIMIT)
        .await
        .map_err(|_| generic_error())
}

fn generic_error() -> String {
    "validation des identifiants impossible".to_string()
}

#[derive(Deserialize)]
struct TelegramResponse {
    ok: bool,
    result: Option<Identity>,
}

#[derive(Deserialize)]
struct Identity {
    id: String,
}

#[derive(Deserialize)]
struct SlackResponse {
    ok: bool,
    user_id: Option<String>,
    url: Option<String>,
}

#[derive(Deserialize)]
struct DiscordResponse {
    id: String,
}

#[cfg(test)]
#[path = "token_probe_tests.rs"]
mod tests;
