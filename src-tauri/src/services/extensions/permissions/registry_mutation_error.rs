use super::OperationFailure;

pub trait MutationError {
    fn storage() -> Self;
}

impl MutationError for String {
    fn storage() -> Self {
        super::error_codes::REGISTRY_UNAVAILABLE.to_string()
    }
}

impl MutationError for OperationFailure {
    fn storage() -> Self {
        OperationFailure::StorageFailed
    }
}
