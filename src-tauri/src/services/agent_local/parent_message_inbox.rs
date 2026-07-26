use super::types_ollama::ChatMessage;
use std::collections::VecDeque;
use tokio::sync::{watch, Mutex};

const MAX_QUEUED_BATCHES: usize = 8;
const MAX_MESSAGES_PER_BATCH: usize = 16;
const MAX_BATCH_CHARS: usize = 16 * 1024 * 1024;

pub struct ParentMessageInbox {
    state: Mutex<InboxState>,
    signal: watch::Sender<u64>,
}

struct InboxState {
    accepting: bool,
    batches: VecDeque<Vec<ChatMessage>>,
}

impl ParentMessageInbox {
    pub fn new() -> Self {
        let (signal, _) = watch::channel(0);
        Self {
            state: Mutex::new(InboxState {
                accepting: true,
                batches: VecDeque::new(),
            }),
            signal,
        }
    }

    pub async fn enqueue(&self, messages: Vec<ChatMessage>) -> Result<bool, String> {
        validate_batch(&messages)?;
        let mut state = self.state.lock().await;
        if !state.accepting {
            return Ok(false);
        }
        if state.batches.len() >= MAX_QUEUED_BATCHES {
            return Err("File de messages pleine".into());
        }
        state.batches.push_back(messages);
        self.signal.send_modify(|sequence| *sequence = sequence.wrapping_add(1));
        Ok(true)
    }

    pub async fn drain_into(&self, messages: &mut Vec<ChatMessage>) -> usize {
        let mut state = self.state.lock().await;
        let count = state.batches.len();
        while let Some(batch) = state.batches.pop_front() {
            messages.extend(batch);
        }
        count
    }

    pub async fn finish_or_drain(&self, messages: &mut Vec<ChatMessage>) -> bool {
        let mut state = self.state.lock().await;
        if state.batches.is_empty() {
            state.accepting = false;
            return false;
        }
        while let Some(batch) = state.batches.pop_front() {
            messages.extend(batch);
        }
        true
    }

    pub fn subscribe(&self) -> watch::Receiver<u64> {
        self.signal.subscribe()
    }

    pub async fn close(&self) {
        self.state.lock().await.accepting = false;
    }
}

fn validate_batch(messages: &[ChatMessage]) -> Result<(), String> {
    if messages.is_empty() || messages.len() > MAX_MESSAGES_PER_BATCH {
        return Err("Message invalide".into());
    }
    let mut chars = 0usize;
    for message in messages {
        if message.role != "user"
            || message.tool_calls.is_some()
            || message.tool_name.is_some()
            || message.tool_call_id.is_some()
            || message.reasoning_content.is_some()
        {
            return Err("Message invalide".into());
        }
        chars = chars.saturating_add(message.content.chars().count());
        let images = message.images.as_deref().unwrap_or_default();
        if images.len() > 8 {
            return Err("Message invalide".into());
        }
        chars = images
            .iter()
            .fold(chars, |total, image| total.saturating_add(image.chars().count()));
        if chars > MAX_BATCH_CHARS {
            return Err("Message invalide".into());
        }
    }
    Ok(())
}
