use super::discord::DiscordAdapter;
use super::discord_types::DiscordMessage;
use super::{GatewayError, GatewayResult, InboundMessage};
use crate::services::api_keys;
use crate::services::gateway::types::ChannelKey;

impl DiscordAdapter {
    pub(super) async fn load_token(&self, vault_key: &str) -> GatewayResult<()> {
        let token = api_keys::get_raw(vault_key)
            .map_err(|_| GatewayError::auth("token Discord manquant dans le vault"))?;
        self.state.write().await.bot_token = Some(token);
        Ok(())
    }

    pub(super) fn to_inbound(
        msg: &DiscordMessage,
        key: &ChannelKey,
        require_mention: bool,
        bot_user_id: &str,
    ) -> Option<InboundMessage> {
        if msg.is_from_bot() || msg.content.is_empty() {
            return None;
        }
        let is_group = !msg.is_dm();
        if is_group && require_mention && !msg.mentions_user(bot_user_id) {
            return None;
        }
        Some(InboundMessage {
            channel_key: key.clone(),
            user_id: msg.author.id.clone(),
            content: msg.content.clone(),
            message_id: msg.id.clone(),
            chat_id: msg.channel_id.clone(),
            thread_id: None,
            is_group,
            mentions_bot: msg.mentions_user(bot_user_id),
        })
    }
}
