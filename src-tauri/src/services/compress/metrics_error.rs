use serde::Serialize;

use super::checkpoint_transaction::CompressionError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CompressionMetricPhase {
    Snapshot,
    Summary,
    Candidate,
    Commit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CompressionMetricError {
    Unavailable,
    AutomaticSuspended,
    InvalidSnapshot,
    OpenTurn,
    InvalidSummary,
    InvalidCandidate,
    CapacityExceeded,
    InsufficientReduction,
    PrepareFailed,
    SessionChanged,
    SaveFailed,
}

impl CompressionMetricError {
    pub const fn code(self) -> &'static str {
        match self {
            Self::Unavailable => "unavailable",
            Self::AutomaticSuspended => "automatic_suspended",
            Self::InvalidSnapshot => "invalid_snapshot",
            Self::OpenTurn => "open_turn",
            Self::InvalidSummary => "invalid_summary",
            Self::InvalidCandidate => "invalid_candidate",
            Self::CapacityExceeded => "capacity_exceeded",
            Self::InsufficientReduction => "insufficient_reduction",
            Self::PrepareFailed => "prepare_failed",
            Self::SessionChanged => "session_changed",
            Self::SaveFailed => "save_failed",
        }
    }

    pub const fn phase(self) -> CompressionMetricPhase {
        match self {
            Self::Unavailable | Self::AutomaticSuspended | Self::InvalidSnapshot => {
                CompressionMetricPhase::Snapshot
            }
            Self::InvalidSummary => CompressionMetricPhase::Summary,
            Self::OpenTurn
            | Self::InvalidCandidate
            | Self::CapacityExceeded
            | Self::InsufficientReduction => CompressionMetricPhase::Candidate,
            Self::PrepareFailed | Self::SessionChanged | Self::SaveFailed => {
                CompressionMetricPhase::Commit
            }
        }
    }
}

impl From<CompressionError> for CompressionMetricError {
    fn from(value: CompressionError) -> Self {
        match value {
            CompressionError::Unavailable | CompressionError::UnavailableUnder64K => {
                Self::Unavailable
            }
            CompressionError::AutomaticSuspended => Self::AutomaticSuspended,
            CompressionError::SnapshotInvalid => Self::InvalidSnapshot,
            CompressionError::OpenTurn => Self::OpenTurn,
            CompressionError::SummaryInvalid => Self::InvalidSummary,
            CompressionError::CandidateInvalid => Self::InvalidCandidate,
            CompressionError::CapacityExceeded => Self::CapacityExceeded,
            CompressionError::InsufficientReduction => Self::InsufficientReduction,
            CompressionError::PrepareFailed => Self::PrepareFailed,
            CompressionError::SessionChanged => Self::SessionChanged,
            CompressionError::SaveFailed => Self::SaveFailed,
        }
    }
}
