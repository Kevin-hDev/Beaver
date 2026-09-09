use super::ProviderErrorCode;

// Observed OpenRouter refusal, 2026-09-09. Match only the complete known
// requirement: never expose arbitrary provider text or infer age from a 403.
const AGE_REQUIREMENT: &str = "This model requires you to complete the following before use: 18+ age confirmation. Confirm at https://openrouter.ai/settings/preferences.";

pub(super) fn classify(body: &str) -> ProviderErrorCode {
    let document = serde_json::from_str::<serde_json::Value>(body).ok();
    if document
        .as_ref()
        .and_then(|value| value.pointer("/error/message"))
        .and_then(serde_json::Value::as_str)
        == Some(AGE_REQUIREMENT)
    {
        ProviderErrorCode::ProviderAgeConfirmationRequired
    } else {
        ProviderErrorCode::ProviderAccessUnavailable
    }
}
