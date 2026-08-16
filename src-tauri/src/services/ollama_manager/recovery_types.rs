#![allow(dead_code)]

use super::error::OllamaErrorCode;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RecoveryReason {
    Startup,
    Retry,
    Manual,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RecoveryOutcome {
    Ready,
    ProgressMade,
    Deferred { code: OllamaErrorCode },
}
