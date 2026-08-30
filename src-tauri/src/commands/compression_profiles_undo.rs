use std::time::{Duration, Instant};

use crate::services::compress::profile_store::CompressionProfileStoreError;
use crate::services::compress::profile_store_document::CompressionProfileDocument;

pub(super) const UNDO_DURATION: Duration = Duration::from_secs(30);

#[derive(Default)]
pub(super) struct UndoSlot {
    snapshot: Option<DeletedProfileSnapshot>,
}

struct DeletedProfileSnapshot {
    token: String,
    document_before: CompressionProfileDocument,
    document_after: CompressionProfileDocument,
    expires_at: Instant,
}

impl UndoSlot {
    pub(super) fn record(
        &mut self,
        document_before: CompressionProfileDocument,
        document_after: CompressionProfileDocument,
        now: Instant,
    ) -> String {
        let token = uuid::Uuid::new_v4().to_string();
        self.snapshot = Some(DeletedProfileSnapshot {
            token: token.clone(),
            document_before,
            document_after,
            expires_at: now + UNDO_DURATION,
        });
        token
    }

    pub(super) fn candidate(
        &self,
        token: &str,
        now: Instant,
    ) -> Result<
        (CompressionProfileDocument, CompressionProfileDocument),
        CompressionProfileStoreError,
    > {
        let snapshot = self
            .snapshot
            .as_ref()
            .ok_or(CompressionProfileStoreError::Invalid)?;
        if now > snapshot.expires_at || !constant_time_token_eq(token, &snapshot.token) {
            return Err(CompressionProfileStoreError::Invalid);
        }
        Ok((
            snapshot.document_before.clone(),
            snapshot.document_after.clone(),
        ))
    }

    pub(super) fn clear_if_token(&mut self, token: &str) {
        if self
            .snapshot
            .as_ref()
            .is_some_and(|snapshot| constant_time_token_eq(token, &snapshot.token))
        {
            self.snapshot = None;
        }
    }
}

fn constant_time_token_eq(candidate: &str, expected: &str) -> bool {
    const UUID_TEXT_BYTES: usize = 36;
    let candidate = candidate.as_bytes();
    let expected = expected.as_bytes();
    let mut difference = candidate.len() ^ expected.len();
    for index in 0..UUID_TEXT_BYTES {
        let left = candidate.get(index).copied().unwrap_or_default();
        let right = expected.get(index).copied().unwrap_or_default();
        difference |= usize::from(left ^ right);
    }
    difference == 0
}
