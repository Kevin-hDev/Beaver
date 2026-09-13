use super::checkpoint_candidate::{CompressionCandidate, CompressionSelectionReport};
use crate::services::agent_local::types_ollama::ChatMessage;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompressionError {
    Unavailable,
    UnavailableUnder64K,
    AutomaticSuspended,
    SnapshotInvalid,
    OpenTurn,
    SummaryInvalid,
    SummaryRequestFailed,
    Cancelled,
    CandidateInvalid,
    CapacityExceeded,
    CapacityUnverified,
    InsufficientReduction,
    PrepareFailed,
    SessionChanged,
    SaveFailed,
}

impl CompressionError {
    pub(crate) fn from_code(code: &'static str) -> Self {
        match code {
            crate::services::agent_local::context_capacity_error::CODE => Self::CapacityExceeded,
            crate::services::agent_local::context_capacity_error::UNVERIFIED_CODE => {
                Self::CapacityUnverified
            }
            "compression_checkpoint_invalid" => Self::OpenTurn,
            _ => Self::CandidateInvalid,
        }
    }

    pub const fn public_message(self) -> &'static str {
        match self {
            Self::UnavailableUnder64K => "compression_disabled_under_64k",
            Self::AutomaticSuspended => "compression_automatic_suspended",
            Self::Unavailable => "compression_unavailable",
            Self::CapacityUnverified => {
                crate::services::agent_local::context_capacity_error::UNVERIFIED_CODE
            }
            _ => "compression_failed",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompressionCommitReport {
    pub before_tokens: u32,
    pub after_tokens: u32,
    pub compression_count: u32,
    pub selection: CompressionSelectionReport,
}

pub async fn commit_candidate(
    session_id: &str,
    runtime_messages: &mut Vec<ChatMessage>,
    candidate: CompressionCandidate,
) -> Result<CompressionCommitReport, CompressionError> {
    crate::services::agent_local::session_store::validate_session_id(session_id)
        .map_err(|_| CompressionError::SnapshotInvalid)?;
    let lock = crate::services::agent_local::session_store::lock_session(session_id).await;
    let _guard = lock.lock().await;
    let mut session = crate::services::agent_local::session_store::get(session_id)
        .await
        .map_err(|_| CompressionError::SaveFailed)?;
    if !super::checkpoint_candidate_validation::same_messages(
        &session.messages,
        &candidate.source_messages,
    ) {
        return Err(CompressionError::SessionChanged);
    }
    session.messages = candidate.persisted_messages;
    session.compression_count = session.compression_count.saturating_add(1);
    session.automatic_compression_guard = candidate.automatic_compression_guard;
    session.updated_at = Some(chrono::Utc::now());
    if let Some(preparation) = &mut session.context_usage.current_preparation {
        preparation.input = candidate.prepared_count.clone();
        preparation.transient_overhead_tokens = 0;
        preparation.state =
            crate::services::agent_local::context_usage_record::ContextPreparationState::Ready;
        preparation.updated_at = chrono::Utc::now();
    }
    crate::services::agent_local::session_store_messages::recompute_accumulated_tokens(
        &mut session,
    );
    let prepared = crate::services::agent_local::session_store::prepare_document(&session)
        .await
        .map_err(|_| CompressionError::PrepareFailed)?;
    crate::services::agent_local::session_store::save_prepared(prepared)
        .await
        .map_err(|_| CompressionError::SaveFailed)?;
    *runtime_messages = candidate.runtime_messages;
    Ok(CompressionCommitReport {
        before_tokens: candidate.before_tokens,
        after_tokens: candidate.after_tokens,
        compression_count: session.compression_count,
        selection: candidate.report,
    })
}
