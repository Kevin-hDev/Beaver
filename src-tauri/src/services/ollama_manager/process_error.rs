use super::error::OllamaErrorCode;
use super::process::OllamaProcessError;

pub(crate) fn map_process_error(error: OllamaProcessError) -> OllamaErrorCode {
    match error {
        OllamaProcessError::Receipt => OllamaErrorCode::OllamaStorageUnavailable,
        OllamaProcessError::EmergencyCapacity => OllamaErrorCode::OllamaOperationInProgress,
        OllamaProcessError::Spawn
        | OllamaProcessError::Gate
        | OllamaProcessError::Admission
        | OllamaProcessError::Identity
        | OllamaProcessError::Reap
        | OllamaProcessError::InvalidState => OllamaErrorCode::OllamaStartFailed,
    }
}
