use std::sync::Arc;

use tokio::sync::RwLock;

use super::telegram::{TelegramAdapter, TelegramState};
use super::telegram_types::*;
use super::{GatewayError, GatewayResult, InboundMessage};
use crate::services::api_keys;
use crate::services::gateway::types::ChannelKey;
use crate::services::secure_http::{read_json_bounded, AuthenticatedClient, TELEGRAM_BODY_LIMIT};

impl TelegramAdapter {
    pub(super) async fn load_token_and_identity(&self, vault_key: &str) -> GatewayResult<()> {
        let token = api_keys::get_raw(vault_key)
            .map_err(|_| GatewayError::auth("token Telegram manquant dans le vault"))?;
        let url = Self::api_url(&token, "getMe");
        let request = self.client.get(url.as_str());
        let response = self
            .client
            .send(request)
            .await
            .map_err(|_| GatewayError::network("connexion Telegram impossible"))?;
        if !response.status().is_success() {
            return Err(GatewayError::auth("identifiants Telegram refusés"));
        }
        let resp: TgResponse<TgUser> = read_json_bounded(response, TELEGRAM_BODY_LIMIT)
            .await
            .map_err(|_| GatewayError::network("réponse Telegram invalide"))?;
        if !resp.ok {
            return Err(GatewayError::auth("identifiants Telegram refusés"));
        }
        let bot_user = resp
            .result
            .ok_or_else(|| GatewayError::auth("getMe échoué"))?;
        let mut state = self.state.write().await;
        state.bot_token = Some(token);
        state.bot_username = bot_user.username.unwrap_or_default();
        Ok(())
    }

    pub(super) async fn poll_updates(
        client: &AuthenticatedClient,
        state: &Arc<RwLock<TelegramState>>,
    ) -> GatewayResult<Vec<TgUpdate>> {
        let (url, offset) = {
            let s = state.read().await;
            let token = s
                .bot_token
                .as_ref()
                .ok_or_else(|| GatewayError::auth("pas de token"))?;
            (Self::api_url(token, "getUpdates"), s.last_offset)
        };
        let request = client.get(url.as_str()).query(&[
            ("offset", offset + 1),
            ("timeout", 30),
            ("limit", 100),
        ]);
        let resp = client
            .send(request)
            .await
            .map_err(|_| GatewayError::network("réception Telegram impossible"))?;
        let status = resp.status().as_u16();
        if status == 401 || status == 403 {
            return Err(GatewayError::auth("token Telegram invalide"));
        }
        if status == 409 {
            return Err(GatewayError::network("réception Telegram indisponible"));
        }
        let body: TgResponse<TgUpdates> = read_json_bounded(resp, TELEGRAM_BODY_LIMIT)
            .await
            .map_err(|_| GatewayError::network("réponse Telegram invalide"))?;
        if !body.ok {
            return Err(GatewayError::network("réponse Telegram refusée"));
        }
        let updates = body.result.map(TgUpdates::into_inner).unwrap_or_default();
        if let Some(last) = updates.last() {
            state.write().await.last_offset = last.update_id;
        }
        Ok(updates)
    }

    pub(super) fn to_inbound(
        update: &TgUpdate,
        channel_key: &ChannelKey,
        require_mention: bool,
        bot_username: &str,
    ) -> Option<InboundMessage> {
        let msg = update.message.as_ref()?;
        let text = msg.text.as_ref()?;
        let from = msg.from.as_ref()?;
        if msg.chat.is_group() && require_mention && !msg.has_bot_mention(bot_username) {
            return None;
        }
        Some(InboundMessage {
            channel_key: channel_key.clone(),
            user_id: from.id.to_string(),
            content: text.clone(),
            message_id: msg.message_id.to_string(),
            chat_id: msg.chat.id.to_string(),
            thread_id: msg.message_thread_id.map(|id| id.to_string()),
            is_group: msg.chat.is_group(),
            mentions_bot: msg.has_bot_mention(bot_username),
        })
    }

    pub(super) fn send_body(msg: &super::OutboundMessage) -> serde_json::Value {
        let mut body = serde_json::json!({ "chat_id": msg.chat_id, "text": msg.content });
        if let Some(thread_id) = &msg.thread_id {
            body["message_thread_id"] = serde_json::Value::String(thread_id.clone());
        }
        if let Some(reply_to) = &msg.reply_to {
            body["reply_to_message_id"] = serde_json::Value::String(reply_to.clone());
        }
        body
    }
}

#[cfg(test)]
#[path = "telegram_support_tests.rs"]
mod tests;
