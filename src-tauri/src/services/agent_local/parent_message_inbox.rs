use crate::models::agent_turn_contract::NewUserTurnInput;
use std::collections::VecDeque;
use std::future::Future;
use tokio::sync::{watch, Mutex};

pub const MAX_QUEUED_INTENTIONS: usize = 8;

pub struct ParentMessageInbox {
    state: Mutex<InboxState>,
    signal: watch::Sender<u64>,
}

struct InboxState {
    accepting: bool,
    intentions: VecDeque<NewUserTurnInput>,
}

impl ParentMessageInbox {
    pub fn new() -> Self {
        let (signal, _) = watch::channel(0);
        Self {
            state: Mutex::new(InboxState {
                accepting: true,
                intentions: VecDeque::new(),
            }),
            signal,
        }
    }

    pub async fn enqueue(&self, input: NewUserTurnInput) -> Result<bool, String> {
        super::conversation_input::validate_intention(&input).map_err(|_| generic_error())?;
        let mut state = self.state.lock().await;
        if !state.accepting {
            return Ok(false);
        }
        if state.intentions.len() >= MAX_QUEUED_INTENTIONS {
            return Err(generic_error());
        }
        state.intentions.push_back(input);
        self.signal.send_modify(|sequence| *sequence = sequence.wrapping_add(1));
        Ok(true)
    }

    #[allow(dead_code, reason = "Task 10 calls this only after its durable turn commit")]
    pub async fn admit_one_after_commit<F, Fut, T>(&self, admit: F) -> Result<Option<T>, String>
    where
        F: FnOnce(NewUserTurnInput) -> Fut,
        Fut: Future<Output = Result<T, String>>,
    {
        let mut state = self.state.lock().await;
        let Some(input) = state.intentions.front().cloned() else {
            return Ok(None);
        };
        let admitted = admit(input).await?;
        state.intentions.pop_front();
        Ok(Some(admitted))
    }

    #[allow(dead_code, reason = "queue observability is consumed by Task 10 and tests")]
    pub async fn len(&self) -> usize {
        self.state.lock().await.intentions.len()
    }

    pub fn subscribe(&self) -> watch::Receiver<u64> {
        self.signal.subscribe()
    }

    pub async fn close(&self) {
        self.state.lock().await.accepting = false;
    }
}

fn generic_error() -> String {
    "conversation_admission_failed".to_string()
}
