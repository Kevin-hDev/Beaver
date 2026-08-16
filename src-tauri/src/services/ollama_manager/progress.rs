use super::error::OllamaErrorCode;
use super::types::OllamaProgressStage;
use std::sync::Arc;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OllamaProgressUpdate {
    pub stage: OllamaProgressStage,
    pub completed: u64,
    pub total: u64,
}

pub type OllamaProgressReporter = Arc<dyn Fn(OllamaProgressUpdate) + Send + Sync + 'static>;

pub(super) fn report(
    reporter: Option<&OllamaProgressReporter>,
    update: OllamaProgressUpdate,
) -> Result<(), OllamaErrorCode> {
    let Some(reporter) = reporter else {
        return Ok(());
    };
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| reporter(update)))
        .map_err(|_| OllamaErrorCode::OllamaInternal)
}

pub(super) fn report_stage(
    reporter: Option<&OllamaProgressReporter>,
    stage: OllamaProgressStage,
) -> Result<(), OllamaErrorCode> {
    report(
        reporter,
        OllamaProgressUpdate {
            stage,
            completed: 0,
            total: 0,
        },
    )
}
