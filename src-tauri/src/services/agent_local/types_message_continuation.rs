use serde::Deserialize;

use crate::services::reasoning_continuity::envelope::ReasoningEnvelope;

pub(super) fn deserialize<'de, D>(deserializer: D) -> Result<Option<ReasoningEnvelope>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value = Option::<serde_json::Value>::deserialize(deserializer)?;
    Ok(value
        .and_then(|value| serde_json::from_value::<ReasoningEnvelope>(value).ok())
        .filter(|envelope| envelope.validate().is_ok()))
}
