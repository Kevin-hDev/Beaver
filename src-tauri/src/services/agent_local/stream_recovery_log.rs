use std::io::Write;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use tokio_util::sync::CancellationToken;

use super::stream_recovery_record::{
    RecoverableStreamEvent, StreamRecoveryHeader, StreamRecoveryRecord,
};

pub(super) const SYNC_BYTES: u64 = 64 * 1024;

#[derive(Clone)]
pub(crate) struct StreamRecoveryLog {
    pub(super) inner: Arc<Inner>,
}

pub(super) struct Inner {
    pub(super) state: Mutex<State>,
    pub(super) cancel: CancellationToken,
    pub(super) sync_needed: tokio::sync::Notify,
}

pub(super) struct State {
    pub(super) header: StreamRecoveryHeader,
    pub(super) path: PathBuf,
    pub(super) file: Option<std::fs::File>,
    pub(super) bytes: u64,
    pub(super) next_sequence: u64,
    pub(super) dirty_generation: u64,
    pub(super) synced_generation: u64,
    pub(super) dirty_bytes: u64,
    pub(super) sticky_error: Option<String>,
    pub(super) sealed: bool,
}

impl StreamRecoveryLog {
    pub(crate) async fn create(
        header: StreamRecoveryHeader,
        cancel: CancellationToken,
    ) -> Result<(Self, super::stream_recovery_owners::OwnerLease), String> {
        let lease = super::stream_recovery_owners::claim(&header.session_id, &header.request_id)?;
        let (path, file) = super::stream_recovery_store::create(&header).await?;
        let bytes = file.metadata().map_err(|_| error())?.len();
        let inner = Arc::new(Inner {
            state: Mutex::new(State {
                header,
                path,
                file: Some(file),
                bytes,
                next_sequence: 1,
                dirty_generation: 0,
                synced_generation: 0,
                dirty_bytes: 0,
                sticky_error: None,
                sealed: false,
            }),
            cancel,
            sync_needed: tokio::sync::Notify::new(),
        });
        super::stream_recovery_log_sync::start_sync_task(Arc::downgrade(&inner));
        Ok((Self { inner }, lease))
    }

    pub(crate) fn record_event(&self, event: RecoverableStreamEvent) -> Result<(), String> {
        self.append_with(|sequence| StreamRecoveryRecord::Event { sequence, event }, false)
    }

    pub(crate) fn sticky_error(&self) -> Option<String> {
        lock(&self.inner).sticky_error.clone()
    }

    pub(crate) fn path(&self) -> PathBuf {
        lock(&self.inner).path.clone()
    }

    pub(crate) fn header(&self) -> StreamRecoveryHeader {
        lock(&self.inner).header.clone()
    }

    pub(super) fn append_with(
        &self,
        record: impl FnOnce(u64) -> StreamRecoveryRecord,
        force_sync: bool,
    ) -> Result<(), String> {
        let mut state = lock(&self.inner);
        if state.sealed || state.sticky_error.is_some() {
            return Err(error());
        }
        let sequence = state.next_sequence;
        let bytes = match encode(&record(sequence)) {
            Ok(bytes) => bytes,
            Err(_) => return fail(&self.inner, &mut state),
        };
        if state.bytes.saturating_add(bytes.len() as u64)
            > super::stream_recovery_store::MAX_LOG_BYTES
        {
            return fail(&self.inner, &mut state);
        }
        let Some(file) = state.file.as_mut() else {
            return fail(&self.inner, &mut state);
        };
        if file.write_all(&bytes).is_err() {
            return fail(&self.inner, &mut state);
        }
        state.next_sequence += 1;
        state.bytes += bytes.len() as u64;
        state.dirty_bytes += bytes.len() as u64;
        state.dirty_generation += 1;
        if force_sync || state.dirty_bytes >= SYNC_BYTES {
            self.inner.sync_needed.notify_one();
        }
        Ok(())
    }
}

pub(super) fn encode(record: &StreamRecoveryRecord) -> Result<Vec<u8>, String> {
    let mut bytes = serde_json::to_vec(record).map_err(|_| error())?;
    bytes.push(b'\n');
    if bytes.len() > super::stream_recovery_store::MAX_LINE_BYTES {
        return Err(error());
    }
    Ok(bytes)
}

pub(super) fn fail(inner: &Inner, state: &mut State) -> Result<(), String> {
    state.sticky_error = Some(error());
    inner.cancel.cancel();
    Err(error())
}

pub(super) fn lock(inner: &Inner) -> std::sync::MutexGuard<'_, State> {
    inner
        .state
        .lock()
        .unwrap_or_else(|failure| failure.into_inner())
}

pub(super) fn error() -> String {
    "stream_recovery_unavailable".to_string()
}
