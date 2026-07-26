use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct GatewayPayload {
    pub op: u8,
    pub d: Option<serde_json::Value>,
    pub s: Option<u64>,
    pub t: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct Identify<'a> {
    pub op: u8,
    pub d: IdentifyData<'a>,
}

#[derive(Debug, Serialize)]
pub struct IdentifyData<'a> {
    pub token: &'a str,
    pub intents: u32,
    pub properties: IdentifyProperties,
}

#[derive(Debug, Serialize)]
pub struct IdentifyProperties {
    pub os: String,
    pub browser: String,
    pub device: String,
}

#[derive(Debug, Serialize)]
pub struct Heartbeat {
    pub op: u8,
    pub d: Option<u64>,
}

#[derive(Debug, Deserialize)]
pub struct DiscordMessage {
    pub id: String,
    pub channel_id: String,
    pub guild_id: Option<String>,
    pub content: String,
    pub author: DiscordUser,
    pub mentions: Option<Vec<DiscordUser>>,
}

#[derive(Debug, Deserialize)]
pub struct DiscordUser {
    pub id: String,
    pub bot: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct SendMessage {
    pub content: String,
    pub allowed_mentions: AllowedMentions,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message_reference: Option<MessageReference>,
}

#[derive(Debug, Serialize)]
pub struct AllowedMentions {
    pub parse: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct MessageReference {
    pub message_id: String,
}

#[derive(Debug, Deserialize)]
pub struct SentMessage {
    pub id: String,
}

#[derive(Debug, Deserialize)]
pub struct GatewayHello {
    pub heartbeat_interval: u64,
}

#[derive(Debug, Deserialize)]
pub struct ReadyEvent {
    pub user: DiscordUser,
}

pub const GATEWAY_URL: &str = "wss://gateway.discord.gg/?v=10&encoding=json";
pub const INTENT_GUILDS: u32 = 1 << 0;
pub const INTENT_GUILD_MESSAGES: u32 = 1 << 9;
pub const INTENT_DM_MESSAGES: u32 = 1 << 12;
pub const INTENT_MESSAGE_CONTENT: u32 = 1 << 15;

impl DiscordMessage {
    pub fn is_from_bot(&self) -> bool {
        self.author.bot.unwrap_or(false)
    }

    pub fn mentions_user(&self, user_id: &str) -> bool {
        self.mentions
            .as_ref()
            .map(|m| m.iter().any(|u| u.id == user_id))
            .unwrap_or(false)
    }

    pub fn is_dm(&self) -> bool {
        self.guild_id.is_none()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bot_detection() {
        let u = DiscordUser {
            id: "1".into(),
            bot: Some(true),
        };
        let msg = DiscordMessage {
            id: "1".into(),
            channel_id: "c".into(),
            guild_id: None,
            content: "hi".into(),
            author: u,
            mentions: None,
        };
        assert!(msg.is_from_bot());
        assert!(msg.is_dm());
    }

    #[test]
    fn mentions_detection() {
        let author = DiscordUser {
            id: "1".into(),
            bot: None,
        };
        let mentioned = DiscordUser {
            id: "99".into(),
            bot: Some(true),
        };
        let msg = DiscordMessage {
            id: "1".into(),
            channel_id: "c".into(),
            guild_id: Some("g".into()),
            content: "hi <@99>".into(),
            author,
            mentions: Some(vec![mentioned]),
        };
        assert!(msg.mentions_user("99"));
        assert!(!msg.mentions_user("100"));
        assert!(!msg.is_dm());
    }

    #[test]
    fn allowed_mentions_empty_parse() {
        let am = AllowedMentions { parse: vec![] };
        let json = serde_json::to_string(&am).unwrap();
        assert!(json.contains("[]"));
    }
}
