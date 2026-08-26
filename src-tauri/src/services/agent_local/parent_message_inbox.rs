#[cfg(test)]
use crate::models::agent_turn_contract::NewUserTurnInput;
#[cfg(test)]
use std::collections::VecDeque;
#[cfg(test)]
use std::future::Future;
use tokio::sync::{watch, Mutex};

#[cfg(test)]
pub const MAX_QUEUED_INTENTIONS: usize = 8;

pub struct ParentMessageInbox {
    state: Mutex<InboxState>,
    signal: watch::Sender<u64>,
}

struct InboxState {
    accepting: bool,
    #[cfg(test)]
    intentions: VecDeque<NewUserTurnInput>,
}

impl ParentMessageInbox {
    pub fn new() -> Self {
        let (signal, _) = watch::channel(0);
        Self {
            state: Mutex::new(InboxState {
                accepting: true,
                #[cfg(test)]
                intentions: VecDeque::new(),
            }),
            signal,
        }
    }

    #[cfg(test)]
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

    #[cfg(test)]
    pub async fn admit_one_after_commit<F, Fut, T>(&self, admit: F) -> Result<Option<T>, String>
    where
        F: FnOnce(NewUserTurnInput) -> Fut,
        Fut: Future<Output = Result<T, String>>,
    {
        // Cette lease est propre à une inbox et borne une seule admission durable.
        let mut state = self.state.lock().await;
        if !state.accepting {
            return Ok(None);
        }
        let Some(input) = state.intentions.front().cloned() else {
            return Ok(None);
        };
        let admitted = admit(input).await?;
        state.intentions.pop_front();
        Ok(Some(admitted))
    }

    #[cfg(test)]
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

#[cfg(test)]
fn generic_error() -> String {
    "conversation_admission_failed".to_string()
}
