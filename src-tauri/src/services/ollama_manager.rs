mod constants;
mod error;
mod manager;
mod types;

#[cfg(test)]
mod manager_tests;

#[allow(unused_imports)]
pub use error::OllamaErrorCode;
pub use manager::OllamaManager;
#[allow(unused_imports)]
pub use types::{
    BundleState, DaemonState, OllamaEndpoint, OllamaProgressStage, OllamaRuntimeStatus,
    OllamaStartOutcome, OperationState,
};
