use std::sync::{Arc, Weak};

use super::stream_recovery_log::{encode, error, fail, lock, Inner, StreamRecoveryLog};
use super::stream_recovery_record::StreamRecoveryRecord;

impl StreamRecoveryLog {
    pub(crate) async fn stage_messages(
        &self,
        messages: Vec<super::types_message::AgentMessage>,
    ) -> Result<(), String> {
        if messages.is_empty() || messages.len() > super::session_limits::MAX_MESSAGES_PER_SESSION {
            return Err(error());
        }
        let inner = self.inner.clone();
        tokio::task::spawn_blocking(move || rewrite_messages(&inner, messages))
            .await
            .map_err(|_| error())?
    }

    pub(crate) async fn clear_committed(&self) -> Result<(), String> {
        let inner = self.inner.clone();
        tokio::task::spawn_blocking(move || rewrite(&inner, Vec::new()))
            .await
            .map_err(|_| error())?
    }

    pub(crate) async fn stage_turn_ready(&self) -> Result<(), String> {
        self.append_with(
            |sequence| StreamRecoveryRecord::TurnReady { sequence },
            true,
        )?;
        sync_inner(self.inner.clone()).await
    }

    pub(crate) async fn seal_and_remove(&self) -> Result<(), String> {
        sync_inner(self.inner.clone()).await?;
        let path = {
            let mut state = lock(&self.inner);
            state.sealed = true;
            state.file = None;
            state.path.clone()
        };
        super::stream_recovery_store::remove(path).await
    }
}

fn rewrite_messages(
    inner: &Arc<Inner>,
    messages: Vec<super::types_message::AgentMessage>,
) -> Result<(), String> {
    let batch_id = uuid::Uuid::new_v4().to_string();
    let total = messages.len();
    let mut state = lock(inner);
    let mut records = Vec::with_capacity(total);
    for (position, message) in messages.into_iter().enumerate() {
        let sequence = state.next_sequence;
        state.next_sequence += 1;
        records.push(StreamRecoveryRecord::PendingMessage {
            sequence,
            batch_id: batch_id.clone(),
            position,
            total,
            message,
        });
    }
    rewrite_locked(inner, &mut state, records)
}

fn rewrite(inner: &Arc<Inner>, records: Vec<StreamRecoveryRecord>) -> Result<(), String> {
    let mut state = lock(inner);
    rewrite_locked(inner, &mut state, records)
}

fn rewrite_locked(
    inner: &Arc<Inner>,
    state: &mut super::stream_recovery_log::State,
    records: Vec<StreamRecoveryRecord>,
) -> Result<(), String> {
    if state.sealed || state.sticky_error.is_some() {
        return Err(error());
    }
    let mut bytes = match encode(&StreamRecoveryRecord::Header(state.header.clone())) {
        Ok(bytes) => bytes,
        Err(_) => return fail(inner, state),
    };
    for record in records {
        let encoded = match encode(&record) {
            Ok(bytes) => bytes,
            Err(_) => return fail(inner, state),
        };
        bytes.extend(encoded);
    }
    if bytes.len() as u64 > super::stream_recovery_store::MAX_LOG_BYTES {
        return fail(inner, state);
    }
    state.file = None;
    if crate::services::private_store::atomic_write(&state.path, &bytes).is_err() {
        return fail(inner, state);
    }
    state.file = match super::stream_recovery_store::open_append(&state.path) {
        Ok(file) => Some(file),
        Err(_) => return fail(inner, state),
    };
    state.bytes = bytes.len() as u64;
    state.dirty_bytes = 0;
    state.dirty_generation += 1;
    state.synced_generation = state.dirty_generation;
    Ok(())
}

pub(super) fn start_sync_task(inner: Weak<Inner>) {
    tokio::spawn(async move {
        loop {
            let Some(current) = inner.upgrade() else {
                break;
            };
            tokio::select! {
                _ = tokio::time::sleep(std::time::Duration::from_secs(1)) => {}
                _ = current.sync_needed.notified() => {}
            }
            drop(current);
            let Some(current) = inner.upgrade() else {
                break;
            };
            if sync_inner(current).await.is_err() {
                break;
            }
        }
    });
}

async fn sync_inner(inner: Arc<Inner>) -> Result<(), String> {
    let (file, generation) = {
        let state = lock(&inner);
        if state.sealed || state.synced_generation == state.dirty_generation {
            return state.sticky_error.clone().map_or(Ok(()), Err);
        }
        let file = state
            .file
            .as_ref()
            .ok_or_else(error)?
            .try_clone()
            .map_err(|_| error())?;
        (file, state.dirty_generation)
    };
    let result = tokio::task::spawn_blocking(move || file.sync_data()).await;
    let mut state = lock(&inner);
    if !matches!(result, Ok(Ok(()))) {
        return fail(&inner, &mut state);
    }
    if state.dirty_generation == generation {
        state.synced_generation = generation;
        state.dirty_bytes = 0;
    }
    Ok(())
}
