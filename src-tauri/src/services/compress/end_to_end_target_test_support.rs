use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;

use async_trait::async_trait;

use super::super::profile_resolve::resolve_from_document;
use super::super::profile_store_document::CompressionProfileDocument;
use super::super::profile_types::CompressionTrigger;
use super::super::snapshot::CompressionSnapshot;
use super::super::summary_contract::SummaryRawOutput;
use super::super::summary_request::{SummaryAttemptError, SummaryCall, SummaryCollector};
use crate::services::agent_local::types_message::AgentMessage;
use crate::services::agent_local::types_ollama::ChatMessage;

pub(super) struct RecordingCollector {
    calls: AtomicUsize,
    limits: Mutex<Vec<u32>>,
    input_tokens: Mutex<Vec<u32>>,
}

impl RecordingCollector {
    pub(super) fn new() -> Self {
        Self {
            calls: AtomicUsize::new(0),
            limits: Mutex::new(Vec::new()),
            input_tokens: Mutex::new(Vec::new()),
        }
    }

    pub(super) fn calls(&self) -> usize {
        self.calls.load(Ordering::Relaxed)
    }

    pub(super) fn limits(&self) -> Vec<u32> {
        self.limits.lock().unwrap().clone()
    }

    pub(super) fn input_tokens(&self) -> Vec<u32> {
        self.input_tokens.lock().unwrap().clone()
    }
}

#[async_trait]
impl SummaryCollector for RecordingCollector {
    async fn collect(&self, call: &SummaryCall) -> Result<SummaryRawOutput, SummaryAttemptError> {
        self.calls.fetch_add(1, Ordering::Relaxed);
        self.limits.lock().unwrap().push(call.maximum_output_tokens);
        let input = super::super::token_estimate::estimate_textual_request_tokens_for_provider(
            &call.provider,
            &call.messages,
            &[],
        )
        .min(u32::MAX as usize) as u32;
        self.input_tokens.lock().unwrap().push(input);
        Ok(SummaryRawOutput {
            content: format!("<summary>{}</summary>", super::support::summary().content),
            tool_call_count: 0,
            truncated: false,
            cancelled: false,
        })
    }
}

pub(super) async fn stored_session() -> crate::services::agent_local::types_session::AgentSession {
    let mut session = crate::services::agent_local::session_store::create_full(
        "target e2e",
        "fixture",
        "ollama",
        false,
        None,
    )
    .await
    .unwrap();
    session.messages = (0..4)
        .flat_map(|index| {
            let turn = AgentMessage::new_turn_id();
            [
                super::super::checkpoint_messages_tests::message(
                    &turn,
                    "user",
                    format!("exact user {index}"),
                ),
                super::super::checkpoint_messages_tests::message(
                    &turn,
                    "assistant",
                    format!("exact assistant {index}"),
                ),
            ]
        })
        .collect();
    crate::services::agent_local::session_store::save(&session)
        .await
        .unwrap();
    session
}

pub(super) fn snapshot(
    session: &crate::services::agent_local::types_session::AgentSession,
    document: &CompressionProfileDocument,
    window: u64,
    before_tokens: u32,
    head_tokens: u32,
    trigger: CompressionTrigger,
) -> CompressionSnapshot {
    CompressionSnapshot::capture(
        session,
        resolve_from_document(None, document).unwrap(),
        window,
        super::support::capabilities(false),
        trigger,
    )
    .unwrap()
    .with_runtime_context(
        vec![ChatMessage::system("s".repeat(head_tokens as usize * 4))],
        Vec::new(),
        before_tokens,
    )
    .unwrap()
}
