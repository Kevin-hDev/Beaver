use super::durable_fs::OllamaFsError;
#[cfg(test)]
use super::durable_fs::{OllamaFsErrorKind, OllamaFsOperation};
use super::error::OllamaErrorCode;

pub(super) fn durable(context: &'static str, error: OllamaFsError) -> OllamaErrorCode {
    record_durable(context, error);
    OllamaErrorCode::OllamaStorageUnavailable
}

pub(super) fn record_durable(context: &'static str, error: OllamaFsError) {
    ::log::error!(
        "[ollama] durable storage failure context={context} kind={:?} operation={:?} os_code={:?}",
        error.kind(),
        error.operation(),
        error.os_code()
    );
}

pub(super) fn io(
    context: &'static str,
    error: &std::io::Error,
    code: OllamaErrorCode,
) -> OllamaErrorCode {
    record_io(context, error);
    code
}

pub(super) fn record_io(context: &'static str, error: &std::io::Error) {
    ::log::error!(
        "[ollama] storage inspection failure context={context} kind={:?} os_code={:?}",
        error.kind(),
        error.raw_os_error()
    );
}

pub(super) fn record_classification(context: &'static str, classification: impl std::fmt::Debug) {
    ::log::error!(
        "[ollama] storage classification failure context={context} classification={classification:?}"
    );
}

#[cfg(test)]
pub(super) const fn diagnostic_fields(
    error: OllamaFsError,
) -> (OllamaFsErrorKind, Option<OllamaFsOperation>, Option<u32>) {
    (error.kind(), error.operation(), error.os_code())
}
