//! Helpers de parsing/construction pour `openai_compat.rs`.

use super::types::LlmError;
use crate::services::secure_http::{read_bounded, PROVIDER_ERROR_LIMIT};
use reqwest::Response;

pub(super) use super::openai_compat_model_parser::parse_models_list;

/// Mappe un statut HTTP d'erreur vers un `LlmError` approprié.
/// Le body fournisseur est lu de façon bornée puis remplacé par un code sûr dans les logs.
pub async fn map_error_status(resp: Response, provider_id: &str) -> LlmError {
    let status = resp.status().as_u16();
    match status {
        401 | 403 => LlmError::Unauthorized,
        429 => {
            let retry_after_secs = resp
                .headers()
                .get("retry-after")
                .and_then(|v| v.to_str().ok())
                .and_then(|s| s.parse().ok());
            LlmError::RateLimit { retry_after_secs }
        }
        _ => {
            let body = zeroize::Zeroizing::new(
                read_bounded(resp, PROVIDER_ERROR_LIMIT)
                    .await
                    .map(|bytes| String::from_utf8_lossy(&bytes).into_owned())
                    .unwrap_or_default(),
            );
            let code = super::provider_error::classify_http(provider_id, status, &body);
            let log_code = super::provider_error::safe_log_code(provider_id, status, &body);
            ::log::warn!("[llm] HTTP {status} code={log_code}");
            if status == 402 {
                return LlmError::KnownProvider(code);
            }
            LlmError::Http {
                status,
                message: "erreur serveur provider".into(),
            }
        }
    }
}
