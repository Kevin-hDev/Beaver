use std::sync::Arc;

use super::discord_types::*;
use bytes::Bytes;
use tokio::sync::RwLock;
use tokio_tungstenite::tungstenite::{Message as WsMessage, Utf8Bytes};
use zeroize::Zeroize;

use crate::services::brand::DISPLAY_NAME;

pub(super) struct SecretTextPayload {
    bytes: Option<Bytes>,
}

impl SecretTextPayload {
    pub(super) fn new(text: &str) -> Self {
        Self {
            bytes: Some(Bytes::copy_from_slice(text.as_bytes())),
        }
    }

    pub(super) fn message(&self) -> Result<WsMessage, ()> {
        let bytes = self.bytes.as_ref().cloned().ok_or(())?;
        let text = Utf8Bytes::try_from(bytes).map_err(|_| ())?;
        Ok(WsMessage::Text(text))
    }

    pub(super) fn zeroize_after_send(&mut self) -> bool {
        let Some(bytes) = self.bytes.take() else {
            return true;
        };
        match bytes.try_into_mut() {
            Ok(mut owned) => {
                owned.as_mut().zeroize();
                self.bytes = Some(owned.freeze());
                true
            }
            Err(shared) => {
                self.bytes = Some(shared);
                false
            }
        }
    }

    #[cfg(test)]
    fn bytes_for_test(&self) -> &[u8] {
        self.bytes.as_deref().unwrap_or_default()
    }
}

impl Drop for SecretTextPayload {
    fn drop(&mut self) {
        let _ = self.zeroize_after_send();
    }
}

#[derive(Clone, Default)]
pub struct HeartbeatSequence(Arc<RwLock<Option<u64>>>);

impl HeartbeatSequence {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn update(&self, sequence: u64) {
        *self.0.write().await = Some(sequence);
    }

    pub async fn current(&self) -> Option<u64> {
        *self.0.read().await
    }
}

pub fn build_identify(token: &str) -> Identify<'_> {
    Identify {
        op: 2,
        d: IdentifyData {
            token,
            intents: INTENT_GUILDS
                | INTENT_GUILD_MESSAGES
                | INTENT_DM_MESSAGES
                | INTENT_MESSAGE_CONTENT,
            properties: IdentifyProperties {
                os: "linux".into(),
                browser: DISPLAY_NAME.into(),
                device: DISPLAY_NAME.into(),
            },
        },
    }
}

#[cfg(test)]
mod tests {
    use super::{HeartbeatSequence, SecretTextPayload};

    #[tokio::test]
    async fn heartbeat_reads_the_latest_sequence() {
        let sequence = HeartbeatSequence::new();
        assert_eq!(sequence.current().await, None);
        sequence.update(42).await;
        assert_eq!(sequence.current().await, Some(42));
    }

    #[test]
    fn discord_secret_payload_zeroizes_the_websocket_allocation() {
        let mut payload = SecretTextPayload::new("{\"token\":\"secret\"}");
        let message = payload.message().expect("valid UTF-8 payload");
        drop(message);

        assert!(payload.zeroize_after_send());
        assert!(payload.bytes_for_test().iter().all(|byte| *byte == 0));
    }
}
