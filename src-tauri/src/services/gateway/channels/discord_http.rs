use zeroize::Zeroizing;

use super::discord::DiscordAdapter;
use super::discord_types::{AllowedMentions, MessageReference, SendMessage, SentMessage};
use super::{GatewayError, GatewayResult, OutboundMessage};
use crate::services::secure_http::{read_bounded, DISCORD_BODY_LIMIT};

pub(super) fn validate_sent_message(status: u16, body: &[u8]) -> GatewayResult<()> {
    if !(200..300).contains(&status) {
        return Err(GatewayError::network("envoi Discord refusé"));
    }
    let sent: SentMessage = serde_json::from_slice(body)
        .map_err(|_| GatewayError::network("réponse Discord invalide"))?;
    if sent.id.is_empty() {
        return Err(GatewayError::network("réponse Discord invalide"));
    }
    Ok(())
}

impl DiscordAdapter {
    pub(super) async fn send_message(&self, msg: OutboundMessage) -> GatewayResult<()> {
        let token = {
            let state = self.state.read().await;
            state
                .bot_token
                .clone()
                .ok_or_else(|| GatewayError::auth("token Discord absent"))?
        };
        let url = format!(
            "https://discord.com/api/v10/channels/{}/messages",
            msg.chat_id
        );
        let body = SendMessage {
            content: msg.content,
            allowed_mentions: AllowedMentions { parse: vec![] },
            message_reference: msg.reply_to.map(|id| MessageReference { message_id: id }),
        };
        let auth = Zeroizing::new(format!("Bot {}", token.as_str()));
        let request = self
            .client
            .post(url)
            .header("Authorization", auth.as_str())
            .json(&body);
        let response = self
            .client
            .send(request)
            .await
            .map_err(|_| GatewayError::network("envoi Discord impossible"))?;
        let status = response.status().as_u16();
        let bytes = read_bounded(response, DISCORD_BODY_LIMIT)
            .await
            .map_err(|_| GatewayError::network("réponse Discord invalide"))?;
        validate_sent_message(status, &bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::validate_sent_message;

    #[test]
    fn rejects_error_json_even_when_it_is_valid_json() {
        assert!(validate_sent_message(401, br#"{"message":"401: Unauthorized"}"#).is_err());
    }

    #[test]
    fn requires_a_non_empty_message_id() {
        assert!(validate_sent_message(200, br#"{}"#).is_err());
        assert!(validate_sent_message(200, br#"{"id":""}"#).is_err());
        assert!(validate_sent_message(200, br#"{"id":"m1"}"#).is_ok());
    }
}
