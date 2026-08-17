use super::error::OllamaErrorCode;
use super::fingerprint::BundleFingerprint;
use crate::services::paths::OllamaPaths;
use async_trait::async_trait;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RecoveryProbeResult {
    Valid,
    Invalid(OllamaErrorCode),
    Deferred(OllamaErrorCode),
}

#[async_trait]
pub trait RecoveryProbe: Send + Sync {
    async fn validate(
        &self,
        target: &BundleFingerprint,
        paths: &OllamaPaths,
    ) -> RecoveryProbeResult;
}
