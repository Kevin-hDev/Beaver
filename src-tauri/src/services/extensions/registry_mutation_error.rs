use super::OperationFailure;

pub trait MutationError {
    fn storage() -> Self;
}

impl MutationError for String {
    fn storage() -> Self {
        "Registre d'extensions indisponible.".to_string()
    }
}

impl MutationError for OperationFailure {
    fn storage() -> Self {
        OperationFailure::StorageFailed
    }
}
