use std::collections::HashMap;
use std::sync::LazyLock;

use tokio::sync::{oneshot, Mutex};
use tokio_util::sync::CancellationToken;

use super::stream_events::AgentEventEmitter;
use super::types_interactive::{AgentInteractiveAnswer, AgentInteractiveQuestion};

const MAX_PENDING: usize = 64;

struct PendingChoice {
    session_id: String,
    questions: Vec<AgentInteractiveQuestion>,
    tx: oneshot::Sender<Vec<AgentInteractiveAnswer>>,
}

static PENDING: LazyLock<Mutex<HashMap<String, PendingChoice>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

pub async fn request(
    on_event: &AgentEventEmitter,
    session_id: &str,
    questions: Vec<AgentInteractiveQuestion>,
    cancel: CancellationToken,
) -> Result<Vec<AgentInteractiveAnswer>, String> {
    let id = uuid::Uuid::new_v4().to_string();
    let (tx, rx) = oneshot::channel();
    {
        let mut pending = PENDING.lock().await;
        if pending.len() >= MAX_PENDING {
            return Err("demande interactive indisponible".into());
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
    super::tool_interactive::emit_request(on_event, session_id.to_string(), id.clone(), questions);

    tokio::select! {
        res = rx => res.map_err(|_| "demande interactive annulée".to_string()),
        _ = cancel.cancelled() => {
            PENDING.lock().await.remove(&id);
            Err("demande interactive annulée".into())
        }
    }
}

pub async fn respond(
    session_id: &str,
    id: &str,
    answers: Vec<AgentInteractiveAnswer>,
) -> Result<(), String> {
    let mut pending_map = PENDING.lock().await;
    let Some(pending) = pending_map.remove(id) else {
        return Err("demande interactive inconnue".into());
    };
    if pending.session_id != session_id {
        pending_map.insert(id.to_string(), pending);
        return Err("demande interactive inconnue".into());
    }
    drop(pending_map);
    let answers = super::tool_interactive_parse::validate_answers(&pending.questions, answers)?;
    pending
        .tx
        .send(answers)
        .map_err(|_| "demande interactive expirée".to_string())
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
pub async fn clear_pending_for_test() {
    PENDING.lock().await.clear();
}
