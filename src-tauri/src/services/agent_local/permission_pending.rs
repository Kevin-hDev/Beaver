use super::permission_gate::PermissionDecision;
use super::permission_request::PermissionRequest;
use super::stream_events::AgentEventEmitter;
use super::types_ollama::StreamEvent;
use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};
use tokio::sync::oneshot;
use tokio_util::sync::CancellationToken;

const MAX_PENDING: usize = 64;

static PENDING: LazyLock<Mutex<HashMap<String, oneshot::Sender<PermissionDecision>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

struct PendingGuard {
    id: String,
    on_event: AgentEventEmitter,
}

impl Drop for PendingGuard {
    fn drop(&mut self) {
        PENDING
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .remove(&self.id);
        let _ = self.on_event.send(StreamEvent::PermissionClosed {
            id: self.id.clone(),
        });
    }
}

pub(super) async fn wait(
    on_event: &AgentEventEmitter,
    request: PermissionRequest,
    cancel: CancellationToken,
    deadline: Option<std::time::Instant>,
) -> PermissionDecision {
    let id = request.id.clone();
    let (sender, receiver) = oneshot::channel();
    {
        let mut pending = PENDING.lock().unwrap_or_else(|error| error.into_inner());
        if pending.len() >= MAX_PENDING {
            return PermissionDecision::Deny;
        }
        pending.insert(id.clone(), sender);
    }
    let _ = on_event.send(StreamEvent::PermissionRequest(request));
    let _guard = PendingGuard {
        id,
        on_event: on_event.clone(),
    };
    if let Some(deadline) = deadline {
        tokio::select! {
            result = receiver => result.unwrap_or(PermissionDecision::Deny),
            _ = cancel.cancelled() => PermissionDecision::Deny,
            _ = tokio::time::sleep_until(deadline.into()) => PermissionDecision::Deny,
        }
    } else {
        tokio::select! {
            result = receiver => result.unwrap_or(PermissionDecision::Deny),
            _ = cancel.cancelled() => PermissionDecision::Deny,
        }
    }
}

pub(super) fn respond(id: &str, decision: PermissionDecision) -> bool {
    PENDING
        .lock()
        .unwrap_or_else(|error| error.into_inner())
        .remove(id)
        .is_some_and(|sender| sender.send(decision).is_ok())
}

#[cfg(test)]
pub(super) fn contains(id: &str) -> bool {
    PENDING
        .lock()
        .unwrap_or_else(|error| error.into_inner())
        .contains_key(id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn timed_out_confirmation_is_removed_and_cannot_apply_late() {
        let id = "permission-timeout".to_string();
        let cancel = CancellationToken::new();
        let task_id = id.clone();
        let task = tokio::spawn(async move {
            wait(
                &AgentEventEmitter::test("session".into()),
                super::super::permission_request::native(
                    task_id,
                    "bash",
                    &serde_json::json!({}),
                ),
                cancel,
                Some(std::time::Instant::now() + std::time::Duration::from_millis(20)),
            )
            .await
        });
        tokio::task::yield_now().await;
        assert!(contains(&id));
        assert_eq!(task.await.unwrap(), PermissionDecision::Deny);
        assert!(!contains(&id));
        assert!(!respond(&id, PermissionDecision::Allow));
    }
}
