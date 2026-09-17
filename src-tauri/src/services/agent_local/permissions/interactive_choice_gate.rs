use std::collections::HashMap;
use std::sync::LazyLock;

use tokio::sync::{oneshot, Mutex};
use tokio_util::sync::CancellationToken;

use super::stream_events::AgentEventEmitter;
use super::types_interactive::{
    AgentInteractiveAnswer, AgentInteractiveChoiceKind, AgentInteractiveQuestion,
};

const MAX_PENDING: usize = 64;
const CHOICE_CANCELLED: &str = "Interactive choice was cancelled.";
const CHOICE_EXPIRED: &str = "Interactive choice is no longer available.";
const CHOICE_UNAVAILABLE: &str = "Interactive choice is unavailable.";

struct PendingChoice {
    session_id: String,
    questions: Vec<AgentInteractiveQuestion>,
    tx: oneshot::Sender<InteractiveChoiceResponse>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum InteractiveChoiceResponse {
    Answered(Vec<AgentInteractiveAnswer>),
    Dismissed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InteractiveChoiceRequestError {
    Cancelled,
    Unavailable,
}

impl InteractiveChoiceRequestError {
    pub fn message(self) -> &'static str {
        match self {
            Self::Cancelled => CHOICE_CANCELLED,
            Self::Unavailable => CHOICE_UNAVAILABLE,
        }
    }
}

static PENDING: LazyLock<Mutex<HashMap<String, PendingChoice>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

pub async fn request(
    on_event: &AgentEventEmitter,
    session_id: &str,
    kind: AgentInteractiveChoiceKind,
    questions: Vec<AgentInteractiveQuestion>,
    cancel: CancellationToken,
) -> Result<InteractiveChoiceResponse, InteractiveChoiceRequestError> {
    let id = uuid::Uuid::new_v4().to_string();
    let (tx, rx) = oneshot::channel();
    {
        let mut pending = PENDING.lock().await;
        if pending.len() >= MAX_PENDING {
            return Err(InteractiveChoiceRequestError::Unavailable);
        }
        pending.insert(
            id.clone(),
            PendingChoice {
                session_id: session_id.to_string(),
                questions: questions.clone(),
                tx,
            },
        );
    }
    super::tool_interactive::emit_request(
        on_event,
        session_id.to_string(),
        id.clone(),
        kind,
        questions,
    );

    tokio::select! {
        res = rx => res.map_err(|_| InteractiveChoiceRequestError::Cancelled),
        _ = cancel.cancelled() => {
            PENDING.lock().await.remove(&id);
            Err(InteractiveChoiceRequestError::Cancelled)
        }
    }
}

pub async fn respond(
    session_id: &str,
    id: &str,
    answers: Vec<AgentInteractiveAnswer>,
) -> Result<(), String> {
    let mut pending_map = PENDING.lock().await;
    let Some(pending) = pending_map.get(id) else {
        return Err(CHOICE_EXPIRED.into());
    };
    if pending.session_id != session_id {
        return Err(CHOICE_EXPIRED.into());
    }
    let answers = super::tool_interactive_parse::validate_answers(&pending.questions, answers)?;
    let pending = pending_map
        .remove(id)
        .ok_or_else(|| CHOICE_EXPIRED.to_string())?;
    drop(pending_map);
    pending
        .tx
        .send(InteractiveChoiceResponse::Answered(answers))
        .map_err(|_| CHOICE_EXPIRED.to_string())
}

pub async fn dismiss(session_id: &str, id: &str) -> Result<(), String> {
    let mut pending_map = PENDING.lock().await;
    let Some(pending) = pending_map.get(id) else {
        return Err(CHOICE_EXPIRED.into());
    };
    if pending.session_id != session_id {
        return Err(CHOICE_EXPIRED.into());
    }
    let pending = pending_map
        .remove(id)
        .ok_or_else(|| CHOICE_EXPIRED.to_string())?;
    drop(pending_map);
    pending
        .tx
        .send(InteractiveChoiceResponse::Dismissed)
        .map_err(|_| CHOICE_EXPIRED.to_string())
}

#[cfg(test)]
pub async fn pending_len_for_test() -> usize {
    PENDING.lock().await.len()
}

#[cfg(test)]
pub async fn fill_pending_for_test(count: usize) {
    let mut pending = PENDING.lock().await;
    pending.clear();
    for index in 0..count {
        let (tx, _rx) = oneshot::channel();
        pending.insert(
            format!("test-{index}"),
            PendingChoice {
                questions: vec![],
                tx,
                session_id: "test-session".to_string(),
            },
        );
    }
}

#[cfg(test)]
pub async fn insert_pending_for_test(id: &str, session_id: &str) {
    let (tx, _rx) = oneshot::channel();
    PENDING.lock().await.insert(
        id.to_string(),
        PendingChoice {
            session_id: session_id.to_string(),
            questions: vec![],
            tx,
        },
    );
}

#[cfg(test)]
pub async fn insert_pending_receiver_for_test(
    id: &str,
    session_id: &str,
) -> oneshot::Receiver<InteractiveChoiceResponse> {
    let (tx, rx) = oneshot::channel();
    PENDING.lock().await.insert(
        id.to_string(),
        PendingChoice {
            session_id: session_id.to_string(),
            questions: vec![],
            tx,
        },
    );
    rx
}

#[cfg(test)]
pub async fn clear_pending_for_test() {
    PENDING.lock().await.clear();
}

#[cfg(test)]
#[path = "interactive_choice_gate_tests.rs"]
mod tests;
