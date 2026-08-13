use super::discord::DiscordState;
use super::discord_events::{handle_gateway_message, DiscordEventOutcome, GatewayMessageContext};
use super::discord_gateway::HeartbeatSequence;
use crate::services::gateway::refusal_audit::RefusalAudit;
use crate::services::gateway::types::ChannelKey;
use futures_util::Sink;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use tokio::sync::{mpsc, RwLock};
use tokio_tungstenite::tungstenite::Message as WsMessage;

#[derive(Default)]
struct TestSink {
    fail_send: bool,
    sent: Vec<WsMessage>,
}

impl Sink<WsMessage> for TestSink {
    type Error = ();

    fn poll_ready(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn start_send(mut self: Pin<&mut Self>, item: WsMessage) -> Result<(), Self::Error> {
        if self.fail_send {
            Err(())
        } else {
            self.sent.push(item);
            Ok(())
        }
    }

    fn poll_flush(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn poll_close(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }
}

fn context<'a>(
    state: &'a Arc<RwLock<DiscordState>>,
    key: &'a ChannelKey,
    sender: &'a mpsc::Sender<super::InboundMessage>,
    audit: &'a RefusalAudit,
    sequence: &'a HeartbeatSequence,
) -> GatewayMessageContext<'a> {
    GatewayMessageContext {
        state,
        key,
        require_mention: false,
        token: "secret-token",
        sender,
        refusal_audit: audit,
        sequence,
    }
}

fn discord_state() -> Arc<RwLock<DiscordState>> {
    Arc::new(RwLock::new(DiscordState {
        bot_token: None,
        bot_user_id: "bot".into(),
    }))
}

#[tokio::test]
async fn failed_identify_send_requests_local_reconnection() {
    let state = discord_state();
    let key = ChannelKey::new("discord", "main");
    let (sender, _receiver) = mpsc::channel(1);
    let (audit, _audit_receiver) = RefusalAudit::channel();
    let sequence = HeartbeatSequence::new();
    let context = context(&state, &key, &sender, &audit, &sequence);
    let mut heartbeat = None;
    let mut sink = TestSink {
        fail_send: true,
        sent: Vec::new(),
    };

    let outcome = handle_gateway_message(
        r#"{"op":10,"d":{"heartbeat_interval":45000}}"#,
        &context,
        &mut heartbeat,
        &mut sink,
    )
    .await;

    assert_eq!(outcome, DiscordEventOutcome::Reconnect);
}

#[tokio::test]
async fn retained_identify_frame_does_not_destroy_a_healthy_connection() {
    let state = discord_state();
    let key = ChannelKey::new("discord", "main");
    let (sender, _receiver) = mpsc::channel(1);
    let (audit, _audit_receiver) = RefusalAudit::channel();
    let sequence = HeartbeatSequence::new();
    let context = context(&state, &key, &sender, &audit, &sequence);
    let mut heartbeat = None;
    let mut sink = TestSink::default();

    let outcome = handle_gateway_message(
        r#"{"op":10,"d":{"heartbeat_interval":45000}}"#,
        &context,
        &mut heartbeat,
        &mut sink,
    )
    .await;

    assert_eq!(outcome, DiscordEventOutcome::Continue);
    assert_eq!(sink.sent.len(), 1);
}

#[tokio::test]
async fn closed_consumer_stops_the_discord_gateway() {
    let state = discord_state();
    let key = ChannelKey::new("discord", "main");
    let (sender, receiver) = mpsc::channel(1);
    drop(receiver);
    let (audit, _audit_receiver) = RefusalAudit::channel();
    let sequence = HeartbeatSequence::new();
    let context = context(&state, &key, &sender, &audit, &sequence);
    let mut heartbeat = None;
    let mut sink = TestSink::default();

    let outcome = handle_gateway_message(
        r#"{"op":0,"t":"MESSAGE_CREATE","d":{"id":"message","channel_id":"chat","guild_id":null,"content":"hello","author":{"id":"user","bot":false},"mentions":[]}}"#,
        &context,
        &mut heartbeat,
        &mut sink,
    )
    .await;

    assert_eq!(outcome, DiscordEventOutcome::ConsumerClosed);
}
