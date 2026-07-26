use std::sync::Arc;
use std::time::Duration;

use super::discord_types::*;
use futures_util::SinkExt;
use tokio::sync::{Mutex, RwLock};
use tokio_tungstenite::tungstenite::Message as WsMessage;

use crate::services::brand::DISPLAY_NAME;

pub type WsSink = futures_util::stream::SplitSink<
    tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>,
    WsMessage,
>;

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

pub async fn heartbeat_loop(
    sink: Arc<Mutex<WsSink>>,
    cancel: tokio_util::sync::CancellationToken,
    interval: Duration,
    sequence: HeartbeatSequence,
) {
    loop {
        tokio::select! {
            _ = cancel.cancelled() => break,
            _ = tokio::time::sleep(interval) => {
                let hb = Heartbeat { op: 1, d: sequence.current().await };
                let json = serde_json::to_string(&hb).unwrap_or_default();
                if sink.lock().await.send(WsMessage::Text(json.into())).await.is_err() {
                    break;
                }
            }
        }
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
    use super::HeartbeatSequence;

    #[tokio::test]
    async fn heartbeat_reads_the_latest_sequence() {
        let sequence = HeartbeatSequence::new();
        assert_eq!(sequence.current().await, None);
        sequence.update(42).await;
        assert_eq!(sequence.current().await, Some(42));
    }
}
