use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct SlackSocketMessage {
    #[serde(rename = "type")]
    pub msg_type: String,
    pub envelope_id: Option<String>,
    pub payload: Option<SlackEventPayload>,
}

#[derive(Debug, Deserialize)]
pub struct SlackEventPayload {
    pub event: Option<SlackEvent>,
}

#[derive(Debug, Deserialize)]
pub struct SlackEvent {
    #[serde(rename = "type")]
    pub event_type: String,
    pub text: Option<String>,
    pub user: Option<String>,
    pub channel: Option<String>,
    pub ts: Option<String>,
    pub thread_ts: Option<String>,
    pub bot_id: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SlackAck {
    pub envelope_id: String,
}

#[derive(Debug, Deserialize)]
pub struct SlackSocketUrl {
    pub ok: bool,
    pub url: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SlackPostResponse {
    pub ok: bool,
    pub ts: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SlackAuthResponse {
    pub ok: bool,
    pub user_id: Option<String>,
}

impl SlackEvent {
    pub fn is_bot_message(&self) -> bool {
        self.bot_id.is_some()
    }

    pub fn is_user_message(&self) -> bool {
        self.event_type == "message" && !self.is_bot_message() && self.text.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bot_message_detected() {
        let evt = SlackEvent {
            event_type: "message".into(),
            text: Some("hi".into()),
            user: Some("U1".into()),
            channel: Some("C1".into()),
            ts: Some("123.456".into()),
            thread_ts: None,
            bot_id: Some("B1".into()),
        };
        assert!(evt.is_bot_message());
        assert!(!evt.is_user_message());
    }

    #[test]
    fn user_message_detected() {
        let evt = SlackEvent {
            event_type: "message".into(),
            text: Some("hello".into()),
            user: Some("U1".into()),
            channel: Some("C1".into()),
            ts: Some("123.456".into()),
            thread_ts: None,
            bot_id: None,
        };
        assert!(!evt.is_bot_message());
        assert!(evt.is_user_message());
    }

    #[test]
    fn ack_serializes() {
        let ack = SlackAck {
            envelope_id: "abc123".into(),
        };
        let json = serde_json::to_string(&ack).unwrap();
        assert!(json.contains("abc123"));
    }
}
