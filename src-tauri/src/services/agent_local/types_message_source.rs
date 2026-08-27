use serde::Deserialize;

use crate::services::reasoning_continuity::envelope::ReasoningSource;

pub(super) fn deserialize<'de, D>(deserializer: D) -> Result<Option<ReasoningSource>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value = Option::<serde_json::Value>::deserialize(deserializer)?;
    Ok(value
        .and_then(|value| serde_json::from_value::<ReasoningSource>(value).ok())
        .filter(|source| source.validate().is_ok()))
}
