use super::durable_fs::OllamaFsError;
#[cfg(test)]
use super::durable_fs::{OllamaFsErrorKind, OllamaFsOperation};
use super::error::OllamaErrorCode;

pub(super) fn durable(context: &'static str, error: OllamaFsError) -> OllamaErrorCode {
    ::log::error!(
        "[ollama] durable storage failure context={context} kind={:?} operation={:?} os_code={:?}",
        error.kind(),
        error.operation(),
        error.os_code()
    );
    OllamaErrorCode::OllamaStorageUnavailable
}

pub(super) fn io(
    context: &'static str,
    error: &std::io::Error,
    code: OllamaErrorCode,
) -> OllamaErrorCode {
    ::log::error!(
        "[ollama] storage inspection failure context={context} kind={:?} os_code={:?}",
        error.kind(),
        error.raw_os_error()
    );
    code
}

#[cfg(test)]
pub(super) const fn diagnostic_fields(
    error: OllamaFsError,
) -> (OllamaFsErrorKind, Option<OllamaFsOperation>, Option<u32>) {
    (error.kind(), error.operation(), error.os_code())
}
