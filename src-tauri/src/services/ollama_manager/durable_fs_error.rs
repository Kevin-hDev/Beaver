use std::fmt;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::services::ollama_manager) enum OllamaFsErrorKind {
    NotFound,
    AlreadyExists,
    SharingViolation,
    PermissionDenied,
    InvalidInput,
    Other,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::services::ollama_manager) enum OllamaFsOperation {
    InspectHandle,
    OpenRoot,
    EnumerateDirectory,
    OpenChild,
    MarkChildDeleted,
    MarkRootDeleted,
    SyncParent,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::services::ollama_manager) struct OllamaFsError {
    kind: OllamaFsErrorKind,
    cancelled: bool,
    os_code: Option<u32>,
    operation: Option<OllamaFsOperation>,
}

impl OllamaFsError {
    pub(in crate::services::ollama_manager) const fn new(kind: OllamaFsErrorKind) -> Self {
        Self {
            kind,
            cancelled: false,
            os_code: None,
            operation: None,
        }
    }

    pub(in crate::services::ollama_manager) const fn from_os_code(
        kind: OllamaFsErrorKind,
        os_code: u32,
    ) -> Self {
        Self {
            kind,
            cancelled: false,
            os_code: Some(os_code),
            operation: None,
        }
    }

    pub(in crate::services::ollama_manager) fn from_io(error: &std::io::Error) -> Self {
        Self {
            kind: io_error_kind(error),
            cancelled: false,
            os_code: error
                .raw_os_error()
                .and_then(|code| u32::try_from(code).ok()),
            operation: None,
        }
    }

    pub(in crate::services::ollama_manager) fn cancelled() -> Self {
        Self {
            kind: OllamaFsErrorKind::Other,
            cancelled: true,
            os_code: None,
            operation: None,
        }
    }

    pub(in crate::services::ollama_manager) const fn kind(self) -> OllamaFsErrorKind {
        self.kind
    }

    pub(in crate::services::ollama_manager) const fn is_cancelled(self) -> bool {
        self.cancelled
    }

    pub(in crate::services::ollama_manager) const fn os_code(self) -> Option<u32> {
        self.os_code
    }

    pub(in crate::services::ollama_manager) const fn at(
        mut self,
        operation: OllamaFsOperation,
    ) -> Self {
        self.operation = Some(operation);
        self
    }

    pub(in crate::services::ollama_manager) const fn operation(self) -> Option<OllamaFsOperation> {
        self.operation
    }
}

impl fmt::Display for OllamaFsError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("durable filesystem operation failed")
    }
}

impl std::error::Error for OllamaFsError {}

fn io_error_kind(error: &std::io::Error) -> OllamaFsErrorKind {
    match error.kind() {
        std::io::ErrorKind::NotFound => OllamaFsErrorKind::NotFound,
        std::io::ErrorKind::AlreadyExists => OllamaFsErrorKind::AlreadyExists,
        std::io::ErrorKind::InvalidInput => OllamaFsErrorKind::InvalidInput,
        std::io::ErrorKind::PermissionDenied => OllamaFsErrorKind::PermissionDenied,
        _ => OllamaFsErrorKind::Other,
    }
}
