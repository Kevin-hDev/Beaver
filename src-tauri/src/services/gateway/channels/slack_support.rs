use zeroize::Zeroizing;

use super::slack::SlackAdapter;
use super::slack_types::*;
use super::{GatewayError, GatewayResult, InboundMessage};
use crate::services::api_keys;
use crate::services::gateway::types::ChannelKey;
use crate::services::secure_http::{read_json_bounded, AuthenticatedClient, SLACK_BODY_LIMIT};

impl SlackAdapter {
    pub(super) async fn load_tokens(&self, vault_prefix: &str) -> GatewayResult<()> {
        let bot = api_keys::get_raw(&format!("{vault_prefix}.bot"))
            .map_err(|_| GatewayError::auth("token bot Slack manquant"))?;
        let app = api_keys::get_raw(&format!("{vault_prefix}.app"))
            .map_err(|_| GatewayError::auth("token app Slack manquant"))?;
        let bot_user_id = Self::get_bot_user_id(&self.client, &bot).await?;
        let mut s = self.state.write().await;
        s.bot_token = Some(bot);
        s.app_token = Some(app);
        s.bot_user_id = bot_user_id;
        Ok(())
    }

    async fn get_bot_user_id(
        client: &AuthenticatedClient,
        bot_token: &str,
    ) -> GatewayResult<String> {
        let request = client
            .post("https://slack.com/api/auth.test")
            .bearer_auth(bot_token);
        let response = client
            .send(request)
            .await
            .map_err(|_| GatewayError::network("identité Slack indisponible"))?;
        let resp: SlackAuthResponse = read_json_bounded(response, SLACK_BODY_LIMIT)
            .await
            .map_err(|_| GatewayError::network("réponse Slack invalide"))?;
        if !resp.ok {
            return Err(GatewayError::auth("identité Slack refusée"));
        }
        resp.user_id
            .filter(|value| !value.is_empty())
            .ok_or_else(|| GatewayError::auth("identité Slack absente"))
    }

    pub(super) async fn get_ws_url(
        client: &AuthenticatedClient,
        app_token: &str,
    ) -> GatewayResult<Zeroizing<String>> {
        let request = client
            .post("https://slack.com/api/apps.connections.open")
            .bearer_auth(app_token);
        let response = client
            .send(request)
            .await
            .map_err(|_| GatewayError::network("connexion Slack impossible"))?;
        let resp: SlackSocketUrl = read_json_bounded(response, SLACK_BODY_LIMIT)
            .await
            .map_err(|_| GatewayError::network("réponse Slack invalide"))?;
        if !resp.ok {
            return Err(GatewayError::auth("connexion Slack refusée"));
        }
        resp.url
            .map(Zeroizing::new)
            .ok_or_else(|| GatewayError::auth("url websocket Slack manquante"))
    }

    pub(super) fn to_inbound(
        evt: &SlackEvent,
        key: &ChannelKey,
        require_mention: bool,
        bot_user_id: &str,
    ) -> Option<InboundMessage> {
        if !evt.is_user_message() {
            return None;
        }
        let channel = evt.channel.clone()?;
        let content = evt.text.clone()?;
        let message_id = evt.ts.clone()?;
        let is_group = !channel.starts_with('D');
        let mentions_bot = content.contains(&format!("<@{bot_user_id}>"));
        if is_group && require_mention && !mentions_bot {
            return None;
        }
        Some(InboundMessage {
            channel_key: key.clone(),
            user_id: evt.user.clone()?,
            content,
            message_id: message_id.clone(),
            chat_id: channel,
            thread_id: evt.thread_ts.clone().or(Some(message_id)),
            is_group,
            mentions_bot,
        })
    }

    pub(super) fn post_body(msg: &super::OutboundMessage) -> serde_json::Value {
        let mut body = serde_json::json!({
            "channel": msg.chat_id,
            "text": msg.content,
        });
        if let Some(thread_id) = &msg.thread_id {
            body["thread_ts"] = serde_json::Value::String(thread_id.clone());
        }
        body
    }
}

#[cfg(test)]
#[path = "slack_support_tests.rs"]
mod tests;
